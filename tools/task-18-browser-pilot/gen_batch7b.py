#!/usr/bin/env python3
r"""Generate the Task 18 batch 7B case files (11 migrated targets).

Batch 7B migrates eleven `browser_object_*` targets (145 `#[test]` fns), one of
which is a `#[path]` SUBMODULE CARRIER with 0 top-level `#[test]` fns and 25
across two submodules. There are NO design-spec 5.11 retentions in this batch:
`find_fixture_self_inspection.py` puts the whole unadjudicated set outside it
(and returns 0 hits when run explicitly over the two submodule files, which its
default `browser_*.rs` scan does not reach), and the `.matches(` / `.lines()` /
`.iter().all|any(` census returns zero hits across all thirteen files. The only
`#[path` hit is the carrier's own two `mod` declarations, which U10 governs
rather than 5.11. Both scans were re-derived at dispatch and their commands and
outputs are recorded in the batch report.

WHY A GENERATOR AND NOT ELEVEN HAND-WRITTEN FILES. Same reason batches 5, 6A,
6B and 7A used one: batch 4 shipped cross-file prose divergence that every
per-file gate passed individually, because no gate reads `#` header prose or
`rationale` wording (U8). Every recurring sentence is therefore CALLED from
`batch5_prose`, not retyped; the batch-6A/7A-local blocks this batch also needs
are IMPORTED from those generators rather than copied, which is the same
discipline one level up. This module writes only the PER-FILE spec -- the
program under test, the invocation arithmetic, the assertion inventory and the
`:N` citations -- which is what review has to read.

Nothing under `tools/` or `scripts/` is modified by this batch; this file and
`batch7b_captures.py` are added and everything else is used as it stands.

CITATIONS. Every `:N` below is produced by SEARCHING the source for the
construct at generation time (`gen_batch7a.cite`, or `sub_cite` below for a
construct that lives in a submodule and therefore needs the qualified
`build.rs:N` form `batch5_crosscheck.py` resolves against that submodule). None
is computed by arithmetic and none is carried over from an earlier measurement.

RULE 8 / RULE 9. Two of these sources build a fixture with a `str::replace`
whose needle carries two leading spaces. Neither text is hand-derived: both are
the byte-exact output of executing the real code and they live in
`batch7b_captures.py`, whose docstring records the exact capture procedure.
`assert_frozen_pair` re-checks each capture against its own `.rs` before it is
emitted, so a capture taken before a source edit fails the generator instead of
shipping a program that is no longer the program under test.

RULE 10. No fixture and no harness body in this batch contains a genuine JS
template literal, so no file here declares `[constants] dollar`. That is CHECKED
rather than assumed: `assert_no_template_literals` greps every emitted `[source]`
value and every step `body` and raises on `${`.

RULE 11. Four of these sources make an OR-shaped assertion. `_stream` resolves
the stream against the real binary per output mode -- refusing to answer if the
binary is absent or if a cell is ambiguous -- and `_needle` then picks the FIRST
source-order disjunct that holds on EVERY cell. Both are recorded in the header
and the source's full disjunction sentence is carried into every affected
rationale.

U9. Every exact pin is live-captured from the real built `kali` via
`kali_run.py`, for every cell it is emitted into, and `batch5_prose.
assert_identical` asserts the cells agree with each other AND with the embedded
constant before one pin is emitted. See `_pin`.

Run: python3 gen_batch7b.py [name ...]   (no args = all)
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

from case_emit import fixture_in_fn, fixture_starting, write, source_text  # noqa: E402
from math_shapes import META, envelope_build, envelope_harness  # noqa: E402
from submodules import submodule_paths  # noqa: E402
import batch5_prose as P  # noqa: E402
import batch7b_captures as C  # noqa: E402

from gen_batch6a import (  # noqa: E402
    FAIL_CLOSED_NOTE, check_program, comment_blocks, hdr,
)
from gen_batch7a import (  # noqa: E402
    ARGV_ORDER_BUILD_ONLY, ARGV_ORDER_HARNESS_ONLY, EXTS4,
    FAIL_CLOSED_NON_AXES, FAIL_CLOSED_NON_AXES_BUILD, HARNESS_ENV,
    NO_TEMPLATE_LITERAL, RUNNER_HARNESS_STEP, assert_env_name, assert_fns,
    assert_bodies_identical, assert_no_template_literals, build as emit_checked, cite,
    matrix_block as _matrix_block,
    rule12_none, test_fns, wrap, word,
)

REGISTRY = {}


def target(name):
    def deco(fn):
        REGISTRY[name] = fn
        return fn
    return deco


def rs(name):
    """The source a case file is generated FROM.

    Batch 7B declares no U4 trim-and-keep retention, so no case file's source is
    a pre-trim blob and this is a plain read. The precondition that makes that
    true IS checked, so a later trim cannot silently regenerate a smaller case
    file from a trimmed source.
    """
    text = source_text(name, quiet=True)
    if text.startswith("//!"):
        raise AssertionError(
            f"browser_{name}.rs has a `//!` header -- batch 7B declares no retentions, so "
            "this is an unexpected trim; regenerate from the pre-trim blob instead")
    return text


def submods(name):
    """`{basename: text}` for a `#[path]` carrier's submodules (U10).

    Resolved through `submodules.submodule_paths`, which delegates to
    `audit-case-migration.py`'s own `resolve_path_mods` -- three of this
    project's measurement bugs came from a second implementation of a predicate
    that already existed, and the carrier's inventory is exactly the figure U10
    exists to stop being wrong.
    """
    paths = submodule_paths(os.path.join(TESTS, f"browser_{name}.rs"))
    if not paths:
        raise AssertionError(f"browser_{name}.rs declares no resolvable submodule")
    return {p.name: p.read_text() for p in paths}


def fn_span(text, fn_name):
    """`(first_line, last_line)` of `fn <fn_name>`'s body, 1-based inclusive."""
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
    return text[:brace].count("\n") + 1, text[:i].count("\n") + 1


def cite_in(text, fn_name, snippet, *, occurrence=1, expect=1):
    """`` `<snippet>` (:N) `` scoped to one fn's body.

    Four of this batch's sources spell `assert!(!output.status.success(), ...)`
    in two or more helpers. An unscoped citation resolves to the first of them
    and the citation gate reports it CORRECT -- the construct really is on that
    line -- while the prose is pointing into a helper the case never reached.
    Scoping is what makes the citation mean what the sentence says.
    """
    lo, hi = fn_span(text, fn_name)
    hits = [i + 1 for i, line in enumerate(text.split("\n"))
            if snippet in line and lo <= i + 1 <= hi]
    if len(hits) != expect:
        raise AssertionError(
            f"citation snippet {snippet!r} in `fn {fn_name}`: {len(hits)} match(es) {hits}, "
            f"wanted {expect}")
    return f"`{snippet}` (:{hits[occurrence - 1]})"


def sub_cite(sub, sub_text, snippet, *, occurrence=1, expect=1, within=None):
    """`` `<snippet>` (<sub>.rs:N) `` -- the QUALIFIED citation form.

    A bare `:N` is resolved by `batch5_crosscheck.py` against the CARRIER, where
    a construct living in a submodule does not exist. The qualified form names
    the submodule and is resolved against it.

    `within` scopes the search to one `#[test]` fn, and that is not a
    convenience: `build.rs` spells `output.status.success(),` on seven lines
    across six different tests, so an unscoped citation would resolve to a
    construct in a DIFFERENT test than the case it annotates -- and
    `batch5_crosscheck.py` would report it correct, because the construct really
    is on that line. Only scoping makes the citation mean what the prose says.
    """
    if "`" in snippet or "\n" in snippet:
        raise AssertionError(f"a cited snippet cannot contain a backtick or newline: {snippet!r}")
    lo, hi = fn_span(sub_text, within) if within else (1, len(sub_text.split("\n")))
    hits = [i + 1 for i, line in enumerate(sub_text.split("\n"))
            if snippet in line and lo <= i + 1 <= hi]
    if len(hits) != expect:
        raise AssertionError(
            f"citation snippet {snippet!r} in {sub}: {len(hits)} match(es) {hits}, "
            f"wanted {expect}")
    return f"`{snippet}` ({sub}:{hits[occurrence - 1]})"


# --------------------------------------------------------------------------
# Rule 12 -- prose extraction, with U6's per-helper attribution.
# --------------------------------------------------------------------------

def blocks_in_fn(text, fn_name):
    """Every Rust comment block whose first line falls inside `fn <fn_name>`.

    U6 is explicit that a comment belongs to the cases its PRODUCING HELPER
    reaches and that copying every block into every case to turn
    `comment_coverage.py` green is over-attribution and forbidden. Three of this
    batch's sources carry blocks in more than one helper, so attribution has to
    be computed rather than assumed universal.
    """
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
    lo = text[:brace].count("\n") + 1
    hi = text[:i].count("\n") + 1
    out = [b for b in comment_blocks(text) if lo <= b[0] <= hi]
    if not out:
        raise AssertionError(f"`fn {fn_name}` carries no Rust comment block")
    return out


def distinct_texts(blocks):
    """Each block's lines joined with single spaces, de-duplicated, in order.

    `comment_coverage.py` normalises whitespace and then requires each comment
    LINE's text to appear in each case's rationale, so joining a block's lines
    with single spaces discharges every line of that block at once.
    """
    out = []
    for _start, texts in blocks:
        joined = " ".join(t.strip() for t in texts if t.strip())
        if joined and joined not in out:
            out.append(joined)
    return out


def prose_of(texts, stem):
    """The rule-12 tail appended to a rationale, for a chosen set of blocks."""
    return (f" RULE 12 -- the Rust comment prose of browser_{stem}.rs, carried verbatim: "
            + " ".join(f"\"{t}\"" for t in texts))


def rule12_block(stem, blocks, *, reaching, extra_files=()):
    """The header accounting for a source that DOES carry Rust comments.

    The comment text itself is EXTRACTED, never retyped -- rule 12 is explicit
    that an em-dash retyped as `--` is a violation the mechanical checker
    catches, and one source in this batch carries a real em-dash.
    """
    texts = distinct_texts(blocks)
    lines = [
        "RULE 12 (carry every source comment verbatim) -- THIS SOURCE HAS PROSE, and it is",
        "carried into EVERY case's own `rationale` that its producing helper reaches, not",
        "just into this header.",
        f"`grep -nE '^\\s*//'` over tests/browser_{stem}.rs"
        + (" and its `#[path]` submodules" if extra_files else "")
        + f" returns {sum(len(b[1]) for b in blocks)} Rust comment line(s)",
        f"in {len(blocks)} contiguous block(s), of {len(texts)} distinct text(s):",
    ]
    for text in texts:
        lines.append(f"  * \"{text}\"")
    lines += wrap(reaching, 86)
    lines += [
        "A pointer (\"see the file header\") would not satisfy rule 12: a reader of one failing",
        "trial sees only that trial's `rationale`.",
    ]
    if len(blocks) > len(texts):
        lines += [
            f"The {len(blocks)} blocks are {len(texts)} distinct text(s) repeated; the",
            "rationale below carries each distinct text once. `comment_coverage.py` checks each",
            "comment LINE's text for membership in each rationale, so one copy discharges every",
            "repetition, and N identical copies would be noise rather than coverage.",
        ]
    return lines


UNIVERSAL_HELPER = (
    "Every block sits inside `{helper}`, which every `[[case]]` below reaches, so U6's "
    "bottom-up attribution puts all of it in every rationale: the attribution is per helper "
    "and this helper is universal in this file, which is NOT the over-attribution U6 forbids "
    "(that is copying a block into cases its producing helper does not reach)."
)


def u6_partial_note(pairs, cases):
    """The header disclosure U6 requires when coverage is legitimately partial.

    U6: over-attribution "is forbidden, even though it turns the checker green;
    on such a file the checker's false `missing` must be documented in the
    header instead". So the block is named, the cases it reaches are named, and
    the resulting `comment_coverage.py` red is declared here rather than made to
    go away by copying prose into cases its helper never produced.

    EVERY FIGURE HERE IS DERIVED FROM THE CASES THIS RUN ACTUALLY BUILT (ruling
    15's first answer): the reach of each block is counted by searching the
    emitted rationales for that block's own text, and the total is `len(cases)`.
    A typed count would be a figure an unrelated edit to the case plan could
    move with nothing to catch it.
    """
    texts = [t for _p, t in pairs]
    # IS ANY BLOCK'S TEXT CONTAINED IN ANOTHER'S? Measured, not assumed. Fix round 1 (I-2): this
    # paragraph used to explain the prefix overlap UNCONDITIONALLY, and shipped that explanation
    # into the carrier -- a file with exactly ONE comment block, where nothing is a prefix of
    # anything and the gate's per-line count IS the exact complement of the measured reach. The
    # U6 diagnosis was right and its explanation was false, which is ruling 15's trap one level
    # down. So the clause is emitted only when the overlap is real.
    overlapping = any(a != b and a in b for a in texts for b in texts)
    lines = [
        "U6 -- PROSE ATTRIBUTION IS PARTIAL HERE, AND `comment_coverage.py` GOES RED FOR IT.",
        "This source's comment prose does NOT all sit in one universal helper, so each block is",
        "carried only into the rationales of the cases its producing helper actually reaches.",
        "The figure below is COUNTED from the emitted rationales rather than asserted about",
        "them, and it is deliberately the count `comment_coverage.py` itself would take --",
        "\"rationales whose text CONTAINS this block\", not \"cases this helper produced\".",
    ]
    if overlapping:
        lines += [
            "Those two counts differ here, and the reason is measured rather than assumed: one",
            "block's text is CONTAINED IN another's (the shorter note opens the longer one), so a",
            "rationale carrying the longer block also contains the shorter. The measured count is",
            "the one that predicts the gate:",
        ]
    else:
        lines += [
            "No block's text is contained in any other's here, so for this file the two counts",
            "coincide:",
        ]
    for producer, text in pairs:
        reached = sum(1 for c in cases if text in c["rationale"])
        if not reached or reached == len(cases):
            raise AssertionError(
                f"block {text[:40]!r} reaches {reached} of {len(cases)} cases -- that is not "
                "partial attribution, so this paragraph would be describing a state the file "
                "does not have")
        lines += wrap(f"  * {producer} -> present in {reached} of the {len(cases)} "
                      f"rationales: \"{text}\"", 86)
    lines.append(
        "`comment_coverage.py` requires every comment LINE to appear in EVERY case's rationale,")
    if overlapping:
        # N2: this used to add "the count it prints is not the complement of any one block's
        # reach". FALSE -- `comment_coverage.py` prints `from 4/16` and `from 12/16` on this
        # pair, and 4 and 12 are exactly the complements of the stated reaches 12 and 4. It also
        # contradicted the sentence six lines above saying the measured count predicts the gate.
        # False in both branches, load-bearing in neither, so it is DELETED rather than gated.
        lines.append("so it reports a non-empty MISSING set here and exits 1.")
    else:
        reach = sum(1 for c in cases if texts[0] in c["rationale"])
        lines += [
            "so it reports a non-empty MISSING set here and exits 1 -- and with a single block it",
            f"reports exactly the complement, {len(cases) - reach} of {len(cases)} cases, against",
            "every line of that block.",
        ]
    lines += [
        "That red is expected and is NOT suppressed: U6 forbids copying a block into cases its",
        "producing helper does not reach even though doing so would turn the checker green, and",
        "requires the checker's false `missing` to be documented in the header instead. This is",
        "that documentation.",
    ]
    return lines


# --------------------------------------------------------------------------
# Rule 8 / rule 9 -- captured `str::replace` fixtures.
# --------------------------------------------------------------------------

# The leading-space width the shipped captures were taken at. DECLARED so `assert_frozen_pair`
# can compare it against its own measurement on every run, and so every header that states the
# width interpolates the measurement rather than a typed word.
CAPTURED_NEEDLE_INDENT = 2


def assert_frozen_pair(label, rs_text, plain_cap, frozen_cap, *, plain_literal, replace_fn):
    """Prove a captured frozen/plain pair against the source that produced it.

    Three checks, because a capture is the only thing standing between rule 8/9
    and a hand-typed approximation and a STALE capture reproduces the OLD
    program while every text-level gate stays green:

      1. the PLAIN capture is byte-identical to the literal the lexer extracts
         from the shipped `.rs` -- so the capture run really read this source;
      2. the source really does build the frozen variant by `str::replace` off
         the plain one, and the needle/replacement spelled in the `.rs` are the
         ones the captured pair differs by;
      3. applying that same needle -> replacement to the plain capture yields
         the frozen capture exactly. Not a re-derivation of the fixture (rule 8
         forbids that): the emitted text is the CAPTURE, and this is a
         comparison that fails loudly if the two disagree.
    """
    P.assert_identical(f"{label}: captured plain text vs the shipped literal",
                       plain_cap, plain_literal)
    m = re.search(r"\bfn\s+" + re.escape(replace_fn) + r"\b", rs_text)
    if not m:
        raise AssertionError(f"{label}: no `fn {replace_fn}` in source")
    tail = rs_text[m.end():]
    args = re.search(r'\.replace\(\s*"((?:[^"\\]|\\.)*)"\s*,\s*"((?:[^"\\]|\\.)*)"\s*,?\s*\)',
                     tail, re.S)
    if not args:
        raise AssertionError(f"{label}: `fn {replace_fn}` does not spell a two-argument replace")
    needle = args.group(1).encode().decode("unicode_escape")
    repl = args.group(2).encode().decode("unicode_escape")
    if needle not in plain_cap:
        raise AssertionError(f"{label}: the source's replace needle is not in the plain capture")
    if plain_cap.replace(needle, repl) != frozen_cap:
        raise AssertionError(
            f"{label}: applying the source's own replace to the plain capture does not "
            "reproduce the frozen capture -- one of the two captures is stale")
    # IS THE LEADING WHITESPACE LOAD-BEARING? Measured rather than asserted (fix round 1). What
    # actually forces the capture is that the fixture is not a string literal at all: it is
    # produced by `str::replace` at run time, so there is nothing for the lexer to extract and
    # any other route is hand-derivation (rule 8). The indentation is a secondary hazard, and
    # the answer is RETURNED so the header can state which it is rather than overstating it.
    #
    # FIX ROUND 2 (N4). Fix round 1 relaxed this tripwire from `startswith("  ")` to
    # `startswith(" ")` while both headers went on saying "TWO LEADING SPACES" -- so a 1-space
    # reindent, which used to fire, now passed, and the header kept asserting a width nothing
    # checked. And the header's "occurs exactly once" was emitted unconditionally and never
    # computed: a needle occurring twice passed unchanged. Both are now MEASURED and returned,
    # and the width is compared against the value the captures were taken at (ruling 15's first
    # answer: a declared figure the gate compares against its own output every run).
    lead = len(needle) - len(needle.lstrip(" "))
    if lead != CAPTURED_NEEDLE_INDENT:
        raise AssertionError(
            f"{label}: the replace needle carries {lead} leading space(s), not "
            f"{CAPTURED_NEEDLE_INDENT} -- the source was reindented after the captures were "
            "taken, so at least one capture is stale (and every header stating the width is now "
            "wrong)")
    occurrences = plain_cap.count(needle)
    if occurrences < 1:
        raise AssertionError(f"{label}: the needle does not occur in the plain capture")
    latent = plain_cap.replace(needle.lstrip(), repl.lstrip()) == frozen_cap
    return frozen_cap, (not latent), lead, occurrences


# --------------------------------------------------------------------------
# U5 -- renames, and the mechanical safety check.
# --------------------------------------------------------------------------

def u5_check(source, renamed):
    """U5's safety condition, run rather than asserted (batch 5's check)."""
    return P.assert_rename_is_argv_only(source, renamed, EXTS4)


# --------------------------------------------------------------------------
# Rule 11 -- resolve an OR against the real binary.
# --------------------------------------------------------------------------

def _stream(label, cells):
    """The ONE stream that carries `needle`, resolved live per cell.

    Raises if a cell shows the needle on both streams or neither, or if the
    cells disagree: a disjunction that does not resolve to exactly one stream is
    not narrowable, and guessing would be the weakening rule 11 exists to
    prevent.
    """
    from kali_run import KALI, run_kali
    if not os.path.exists(KALI):
        raise AssertionError(
            f"{KALI} absent -- the rule-11 OR for {label} cannot be resolved by guessing; "
            "build the binary and re-run the generator")
    answers = set()
    for entry, program, args, needle, env in cells:
        _code, out, err, _dir = run_kali({entry: program}, args, env=env)
        in_out, in_err = needle in out.decode(), needle in err.decode()
        if in_out == in_err:
            raise AssertionError(
                f"{label} {entry}: {needle!r} in stdout={in_out} stderr={in_err} -- the OR "
                "does not resolve to exactly one stream")
        answers.add("stdout" if in_out else "stderr")
    if len(answers) != 1:
        raise AssertionError(f"{label}: cells disagree on the carrying stream: {answers}")
    return answers.pop()


def _needle(label, stream, needles, cells):
    """The FIRST source-order disjunct that holds on EVERY cell.

    A four-way OR over (stream x needle) can resolve to one stream and still
    leave BOTH needles true, which is what this batch's three harness ORs do.
    Pinning both would strengthen an OR into an AND; picking one at random is
    not a derivation. So the tie is broken by SOURCE ORDER -- the first disjunct
    the source spells that the real binary confirms everywhere -- which is the
    minimal live-verified strengthening and is reproducible. Returns
    `(needle, also_true)` so the header can disclose what else was observed
    rather than imply the others were false.
    """
    from kali_run import KALI, run_kali
    if not os.path.exists(KALI):
        raise AssertionError(f"{KALI} absent -- {label}'s OR cannot be resolved by guessing")
    holds = {n: True for n in needles}
    for entry, program, args, env in cells:
        _code, out, err, _dir = run_kali({entry: program}, args, env=env)
        text = (out if stream == "stdout" else err).decode()
        for n in needles:
            if n not in text:
                holds[n] = False
    universal = [n for n in needles if holds[n]]
    if not universal:
        raise AssertionError(
            f"{label}: no disjunct holds on every cell -- the OR is not narrowable")
    return universal[0], universal[1:]


def _pin(label, embedded, cells):
    """Re-capture an exact `json.stdout` pin from the real binary for every cell.

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
        code, out, err, _dir = run_kali({entry: program}, args, env={HARNESS_ENV: "node"})
        if code != 0:
            raise AssertionError(f"live capture failed for {label} {entry}: {err!r}")
        captured.append(_json.loads(out)["stdout"])
    return P.assert_identical(f"{label}, live-captured over {len(cells)} cell(s), "
                              "against the embedded constant", embedded, *captured)


# A rule-11 needle is PROGRAM TEXT ("Uncaught Error", "unreachable", "E5506"),
# not a Rust identifier, so it is quoted in prose and never backticked: U8's
# checker treats every backticked lower-case identifier as a cited fn and
# resolves it against the source `.rs`, where `unreachable` does not exist --
# the obvious wording turned that gate red on correct prose. Same collision U5's
# shared block records for `import()`/`require()`.
Q = '"'

RULE11_HEADER_INTRO = [
    "RULE 11 -- THE SOURCE MAKES AN OR-SHAPED CLAIM, AND IT WAS RESOLVED AGAINST THE REAL",
    "BINARY RATHER THAN REPRODUCED OR DROPPED. The case format has no disjunction.",
]


def rule11_block(*, disjunction, cite_text, per_mode, needle, also_true, needles):
    lines = list(RULE11_HEADER_INTRO)
    lines.append(f"  {disjunction}")
    lines.append(f"  -- whose cited construct is {cite_text}.")
    lines.append("Every cell was run against the real built `kali`, and the carrying STREAM is")
    lines.append("unambiguous in each output mode:")
    for mode, stream in per_mode:
        lines.append(f"  * {mode} -> {stream}")
    lines += [
        "The generator refuses to emit if a cell shows the needle on both streams or neither,",
        "or if the cells disagree -- a disjunction that does not resolve to exactly one stream",
        "is not narrowable.",
    ]
    if also_true:
        lines += wrap(
            "THE NEEDLE TIE IS BROKEN BY SOURCE ORDER, and what was observed is disclosed "
            "rather than implied away: on the carrying stream the real binary emits "
            + " and ".join(Q + n + Q for n in needles)
            + ", so more than one disjunct is true. Pinning all of them would turn the "
              "source's OR into an AND, which is a strengthening the source never asked for; "
              "picking one at random is not a derivation. The generator therefore pins the "
              "FIRST disjunct the source spells that holds on EVERY cell -- "
              + Q + needle + Q + " -- and "
              "raises if no disjunct is universal.", 86)
    lines += [
        "This is a PRESENCE claim, so narrowing it is a verified strengthening (every run",
        "satisfying the new assertion satisfies the old); rule 2's asymmetry forbids the same",
        "narrowing for an ABSENCE claim, and none is made here. The source's full disjunction",
        "sentence is carried into every affected rationale, so the narrowing is recorded rather",
        "than silent.",
    ]
    return lines


def rule11_rationale(disjunction, cite_text, stream, needle, also_true):
    out = (f" The source's full disjunction sentence, carried verbatim per rule 11: "
           f"{disjunction}; its cited construct is {cite_text}. The case format has no "
           f"disjunction, so that OR was resolved against the real binary rather than "
           f"reproduced: in this output mode the diagnostic text lands on {stream}, so the "
           f"claim is carried as `{stream}_contains`. Narrowing a PRESENCE claim to the stream "
           "that actually carries it is a verified strengthening -- every run satisfying it "
           "satisfies the original OR.")
    if also_true:
        out += (" More than one disjunct is in fact true on that stream ("
                + ", ".join(Q + n + Q for n in also_true)
                + " as well), so the tie is broken by SOURCE ORDER -- the first disjunct the "
                  "source spells that holds on every cell, " + Q + needle + Q
                + " -- rather than by "
                  "pinning all of them, which would turn the OR into an AND.")
    return out


# --------------------------------------------------------------------------
# Step builders.
# --------------------------------------------------------------------------

_UNSET = object()
# `json_claims=None` means "this source asserts NOTHING about the envelope", and
# it has to be spelled explicitly. Defaulting it would let a json sibling ship
# with no envelope claim by omission rather than by decision -- which is the
# direction that silently drops a claim (rule 1), not the one that invents one.


def harness_cli_step(command, entry, json_output, *, asserts, json_claims=_UNSET):
    argv = (["--output", "json"] if json_output else [])
    argv += [command, "--api", "browser", "--max-threads", "0",
             "--max-spawned-processes", "0", entry]
    step = {"args": argv, "env": {HARNESS_ENV: "node"}}
    if json_output:
        if json_claims is _UNSET:
            raise AssertionError("a json_output step needs its json claims stated (or None)")
        if json_claims is not None:
            step["json"] = json_claims
    step.update(asserts)
    return step


def build_step(entry, json_output, *, asserts, json_claims=_UNSET):
    argv = ["build", "--bundle", "--api", "browser"]
    if json_output:
        argv += ["--output", "json"]
    argv += [entry]
    step = {"args": argv}
    if json_output:
        if json_claims is _UNSET:
            raise AssertionError("a json_output build step needs its json claims stated (or None)")
        if json_claims is not None:
            step["json"] = json_claims
    step.update(asserts)
    return step


def meta_step(bundle_dir):
    return {"kind": "file_json", "path": f"{bundle_dir}/{bundle_dir}.meta.json", "fields": META}


def harness_step(bundle_dir, body, asserts):
    step = {"kind": "browser_bundle_harness", "entry": bundle_dir, "body": body}
    step.update(asserts)
    return step


def extra_ok_block(items):
    """`# EXTRA-OK:` declarations plus the shared preamble `check_extra_claims.py` reads.

    Every entry names one asserted string that appears nowhere in the source and
    says why it is legitimate. A genuinely new one will not be on the list and
    will fail the gate (U14's `extra` direction).
    """
    return list(P.EXTRA_CLAIM_PREAMBLE) + [P.extra_ok(v, why) for v, why in items]


def expanded(keys):
    """Every `${ext}` key as the runner and `check_extra_claims.py` see it."""
    return [k.replace("${ext}", e) for k in keys for e in EXTS4]


# The §5.6 non-axis paragraph for a fail-closed target that DOES carry a content claim.
#
# FIX ROUND 1 (I-1). `gen_batch7a.FAIL_CLOSED_NON_AXES` says "this target asserts nothing but
# process failure, so neither dimension changes what is asserted -- they change the argv", and it
# was applied unconditionally. That is TRUE of three files in this batch and FALSE of two: both
# rule-11 harness targets pin a diagnostic needle, and `json_output` moves it from `stderr` to
# `stdout`, so the output mode changes exactly what is asserted -- as those same headers' own
# RULE 11 block states three paragraphs later. The conclusion (not an axis) was right; the reason
# contradicted the file. Design spec §5.6 gives the correct one, so it is used, and
# `assert_non_axis_reason_matches` below makes applying the wrong one a generator error rather
# than a prose defect nothing reads.
def fail_closed_non_axes_with_claim(rs_text):
    """The §5.6 non-axis paragraph, DERIVED per file rather than imported as a constant.

    FIX ROUND 2 (N1, N3). This was a module constant, and a constant is exactly
    how I-1 arose one round earlier: `gen_batch7a.FAIL_CLOSED_NON_AXES` was
    imported unconditionally into two files it was false for. A shared block that
    asserts anything about its caller's source must therefore DERIVE that
    assertion from the caller's source, so the next batch to import it cannot
    inherit a false clause. Two things were wrong when this was a constant:

      * N1 -- it quoted design spec §5.6's text-vs-JSON sentence and then said
        "both dimensions do exactly that here". Measured from the emitted steps,
        `command` asserts the IDENTICAL key signature in both files: it changes
        argv and the `[source]` entry and nothing else. The quote now attaches
        to `json_output` only, which is what §5.6:469 is about, and
        `assert_non_axis_reason_matches` gates BOTH clauses against the steps.
      * N3 -- it said the `json_output` variation is "a loop INSIDE a `#[test]`
        fn". True of `browser_object_values_spread_harness.rs`, FALSE of
        `browser_object_keys_entries_spread_harness.rs`, which unrolls four
        helper calls per extension with the argument alternating literally. The
        load-bearing conclusion (rule 5, no fn per combination) held for both;
        the mechanism word did not, and nothing derived it. The mechanism is no
        longer named. What IS stated is the property this function actually
        measures -- that no `#[test]` fn is dedicated to an output mode -- and it
        raises if that stops being true, in which case the caller needs a
        different block.
    """
    fns = _local_test_fn_names(rs_text)
    json_prefixed = [f for f in fns if f.startswith("json_")]
    if json_prefixed:
        raise AssertionError(
            f"{len(json_prefixed)} `#[test]` fn(s) carry a `json_` prefix, so this source DOES "
            "have a fn per output mode and the closing sentence of this block would be false: "
            f"{json_prefixed[:3]}")
    return NonAxisBlock([
        "`command` and `json_output` are NOT matrix axes, per rule 7, and their reasons DIFFER --",
        "they are stated separately because measuring them separately is what fix round 2 found",
        "the shared block had got wrong.",
        "`json_output` changes WHAT IS ASSERTED, which is design spec 5.6's own note about",
        "output shape: \"Varying output shape changes both the argv and the assertions\". This",
        "target's only content claim is a single diagnostic needle, and the output mode moves the",
        "stream that carries it (see the RULE 11 block below), so the text and json siblings pin",
        "DIFFERENT keys.",
        "`command` does NOT change what is asserted -- run and test carry the identical assertion",
        "keys here, which the generator checks against the emitted steps. It is not an axis for",
        "the other reason in rule 7: it selects a different `[source]` entry (`main*.<ext>` for",
        "run, `smoke*.test.<ext>` for test), and a `[matrix]` axis substitutes ONE string",
        "uniformly across every case rather than selecting among entries.",
        "Each is written as sibling `[[case]]` entries instead, reached by rule 5 rather than",
        f"rule 6: this source has {len(fns)} `#[test]` fn(s) and NONE of them is dedicated to an",
        "output mode (derived here, and this block refuses to render if one ever is), so the fns",
        "cannot map one-to-one onto the cases. The RULE 6 paragraph below states the split.",
    ], kind=HAS_CLAIM, dims=("command", "json_output"))

# --------------------------------------------------------------------------
# The non-axis paragraph gate (ruling 18).
#
# THREE ROUNDS OF HISTORY, because the ordering below is the conclusion of it and not a
# preference. Round 1 found a shared constant applied to two files it was false for. Round 2
# found that the REPLACEMENT block's third clause was false and ungated, and that rewording it
# had silently disabled an arm through capitalisation. Round 3's adversarial sweep then broke the
# marker matching a third way -- one curly apostrophe silences two arms, an en-dash silences a
# third, a hyphenated reflow silences all of them -- and showed the root cause is structural:
# `if MARKER in text:` cannot distinguish "nothing to check" from "failed to match".
#
# So the gate now runs ruling 18's ordering:
#   1. DERIVE over mark -- `fail_closed_non_axes_with_claim()` derives its claims from the
#      caller's source and refuses to render when they are false. Nothing about that survives a
#      reflow, because it never reads prose.
#   2. DISPATCH ON PROVENANCE -- the arm is selected by WHICH BLOCK WAS CALLED, threaded through
#      `out(non_axis=...)` as a tagged value, not by matching its text.
#   3. NON-MATCH IS AN ERROR -- marker matching is retained only as a secondary consistency
#      check, and it now requires EXACTLY ONE marker to match and to agree with the provenance
#      tag. Every silencing mutation in round 3's sweep becomes a loud generator failure.
#   4. NORMALISE last, and never trust it: a normaliser is only ever a whitelist of the failures
#      already seen. Two rounds of this project's history are two entries on that whitelist.
#
# The comment this replaces defended matching on the claim rather than on identity, "so a header
# assembled from a copy, or reworded downstream, is still checked". That rationale is
# empirically inverted: reworded downstream is exactly what silenced it, twice.
# --------------------------------------------------------------------------

CLAIM_FREE = "claim_free"
HAS_CLAIM = "has_claim"


def _local_test_fn_names(text):
    """`#[test]` fn names, tolerating shapes `gen_batch7a.test_fns` misses.

    Round 3 fold: that regex requires a newline between `#[test]` and `fn`, so
    `#[test] fn x()` on one line and an intervening `#[ignore]` both slip past
    it. No such shape exists in this corpus today, but two things here CONSUME
    the answer -- N3's refusal to render, and the fn count interpolated into the
    block -- and a miss would silently weaken the first and misstate the second.
    """
    return re.findall(r"#\[test\]\s*(?:#\[[^\]]*\]\s*)*fn\s+([a-z0-9_]+)", text)


class NonAxisBlock(list):
    """A non-axis paragraph that knows what it is and which dimensions it names.

    Subclasses `list` so every existing caller -- `matrix_block`, `hdr`,
    `case_emit` -- keeps treating it as the list of header lines it always was,
    while `out()` can read `.kind` and `.dims` off it. That is the provenance
    channel: it travels with the block rather than being re-derived from the
    rendered text.
    """

    def __new__(cls, lines, *, kind, dims):
        self = super().__new__(cls, lines)
        return self

    def __init__(self, lines, *, kind, dims):
        super().__init__(lines)
        if kind not in (CLAIM_FREE, HAS_CLAIM):
            raise AssertionError(f"unknown non-axis block kind {kind!r}")
        for d in dims:
            if d not in ("command", "json_output"):
                raise AssertionError(f"unknown non-axis dimension {d!r}")
        self.kind = kind
        self.dims = tuple(dims)


# The two imported claim-free constants, TAGGED. `gen_batch7a` is not modified (this batch may
# not touch existing tools), so the provenance channel is attached here, at the one place this
# generator uses them. `dims` records which dimensions each block's prose actually contrasts:
# the `_BUILD` variant discusses `json_output` only, because its callers issue `build` alone.
def _norm(text):
    """The one normal form both markers and haystack are held in.

    Space-joined, whitespace-collapsed, lower-cased. Rounds 1 and 2 each shipped
    a gate that matched in only one of these forms and went silent in the other;
    `assert_markers_normalised()` below now makes "the markers are already in
    this form" a checked property rather than, as fix round 2's report claimed it
    was, an unimplemented one (M2).
    """
    return re.sub(r"\s+", " ", str(text)).lower()


_CLAIM_FREE_MARKER = _norm("this target asserts nothing but process")
_HAS_CLAIM_MARKER = _norm("this target's only content claim is a single diagnostic needle")
_COMMAND_ARGV_ONLY_MARKER = _norm(
    "`command` does not change what is asserted -- run and test carry the identical assertion "
    "keys here")


def assert_markers_normalised():
    """M2: the guarantee fix round 2's report CLAIMED and did not implement.

    Its §10.6 said "all three markers are asserted lowercase". No such assertion
    existed -- the markers merely happened to be lowercase. Third consecutive
    round in which the sentence describing the fix was itself the defect, so the
    sentence now has code under it. Cheap, and it runs at import.
    """
    for name, marker in (("_CLAIM_FREE_MARKER", _CLAIM_FREE_MARKER),
                         ("_HAS_CLAIM_MARKER", _HAS_CLAIM_MARKER),
                         ("_COMMAND_ARGV_ONLY_MARKER", _COMMAND_ARGV_ONLY_MARKER)):
        if marker != _norm(marker):
            raise AssertionError(f"{name} is not in the normal form the haystack is held in")
    return True


assert_markers_normalised()

CLAIM_FREE_BLOCK = NonAxisBlock(FAIL_CLOSED_NON_AXES, kind=CLAIM_FREE,
                                dims=("command", "json_output"))
CLAIM_FREE_BUILD_BLOCK = NonAxisBlock(FAIL_CLOSED_NON_AXES_BUILD, kind=CLAIM_FREE,
                                      dims=("json_output",))


# Every §5.4 assertion key except `exit`, plus `file_json`'s `fields`. A step carrying any of
# these makes a CONTENT claim; a step carrying only `exit` does not.
_CONTENT_KEYS = frozenset({
    "stdout", "stdout_contains", "stdout_absent", "stdout_count",
    "stderr", "stderr_contains", "stderr_absent",
    "json", "json_paths", "json_null", "json_count", "fields",
})

# Every subcommand `kali` takes that this family's case files invoke. `_step_command` matches
# against this set rather than taking the first non-flag argv element: the positional heuristic
# it replaces returned "browser" for `["--api", "browser", "run", "main.js"]`, which is correct
# for every argv THIS batch emits and wrong for the first importer that orders its flags
# differently (round 3 fold).
_SUBCOMMANDS = ("build", "run", "test", "check")


def _step_command(step):
    """The subcommand a cli step invokes, matched against the known set."""
    args = step.get("args") or []
    found = [a for a in args if a in _SUBCOMMANDS]
    if not found:
        return None
    return found[0]


def _signature(case):
    """The set of §5.4 CONTENT keys a case asserts, ignoring their values.

    Key-level, deliberately: the question these gates ask is "does this
    dimension change WHAT IS ASSERTED", and that is a question about which keys
    appear, not about their contents. It is also the granularity the review
    measured `command` at.
    """
    return frozenset(k for step in case["steps"] for k in step if k in _CONTENT_KEYS)


def _sigs_by(cases, key):
    out = {}
    for case in cases:
        out.setdefault(key(case), set()).add(_signature(case))
    return out


def _is_json_case(case):
    return any("--output" in (s.get("args") or []) for s in case["steps"])


def assert_non_axis_reason_matches(header, cases, non_axis=None):
    """Check a fail-closed non-axis paragraph against what the file asserts.

    `non_axis` is the block object the caller actually used, or None when the
    file uses the stock §5.6 wording. Arms are selected from IT, not from the
    rendered prose; prose matching survives only as a consistency check that
    raises on disagreement or on a non-match.
    """
    text = _norm(" ".join(str(line) for line in header))
    claims = sorted({k for case in cases for step in case["steps"]
                     for k in step if k in _CONTENT_KEYS})
    hits = [name for name, marker in (("claim_free", _CLAIM_FREE_MARKER),
                                      ("has_claim", _HAS_CLAIM_MARKER))
            if marker in text]

    # --- (3) NON-MATCH IS AN ERROR ------------------------------------------------------
    if non_axis is None:
        if hits:
            raise AssertionError(
                f"the header carries the {hits} marker(s) but no fail-closed non-axis block was "
                "declared to `out(non_axis=...)` -- either it was assembled by hand, or a "
                "sentence elsewhere collides with a marker; both need resolving, because the "
                "gate can no longer tell which arm applies")
        return True
    if len(hits) != 1:
        raise AssertionError(
            f"exactly one of the two non-axis markers must match this header; {len(hits)} did "
            f"({hits}). A zero-match is the failure mode that silenced this gate twice: a curly "
            "apostrophe, an en-dash or a hyphenated reflow inside the marker sentence is enough. "
            "Fix the prose or the marker -- do NOT let a non-match read as nothing to check")
    if hits[0] != non_axis.kind:
        raise AssertionError(
            f"provenance says this header uses the {non_axis.kind!r} block but its prose matches "
            f"the {hits[0]!r} marker -- the two disagree, so one of them is wrong")
    if non_axis.kind == HAS_CLAIM and _COMMAND_ARGV_ONLY_MARKER not in text:
        raise AssertionError(
            "a with-claim block must also carry its `command` clause, and the marker for it did "
            "not match -- the clause was reworded, dropped, or reflowed through its own text")

    # --- (M3) A GATED CLAUSE MAY NOT RENDER OVER A DIMENSION THE FILE DOES NOT HAVE -----
    # Both arms below compare GROUPS, so on a one-sided file they compare nothing and pass
    # vacuously while the prose asserts a contrast. Latent in this batch (every file carrying a
    # block has both sides of both dimensions) and live for the first importer that does not.
    if "command" in non_axis.dims:
        commands = {c for c in (next((_step_command(s) for s in case["steps"] if s.get("args")),
                                     None) for case in cases) if c}
        if len(commands) < 2:
            raise AssertionError(
                f"this block's prose contrasts run and test, but the file issues {sorted(commands)}"
                " -- the clause would be describing a dimension the file does not have")
    if "json_output" in non_axis.dims:
        modes = {_is_json_case(case) for case in cases}
        if len(modes) < 2:
            raise AssertionError(
                "this block's prose contrasts text and `--output json`, but every case is in one "
                "mode -- the clause would be describing a dimension the file does not have")

    # --- (2) THE SUBSTANTIVE ARMS, dispatched on provenance -----------------------------
    if non_axis.kind == CLAIM_FREE:
        if claims:
            raise AssertionError(
                "the header says this target asserts nothing but process failure, but its steps "
                f"carry content claim key(s) {claims} -- use fail_closed_non_axes_with_claim()")
        return True

    if not claims:
        raise AssertionError(
            "the header says this target carries a content claim, but no step carries one -- "
            "use FAIL_CLOSED_NON_AXES")
    # The clause does not merely say a claim EXISTS; it says the OUTPUT MODE MOVES it.
    by_mode = _sigs_by(cases, _is_json_case)
    if len({frozenset(v) for v in by_mode.values()}) == 1:
        raise AssertionError(
            "the header says the output mode moves what is asserted, but the text and json cases "
            f"carry the same assertion keys ({claims})")
    # ... and that `command` does NOT move it.
    by_cmd = _sigs_by(cases, lambda c: next(
        (_step_command(s) for s in c["steps"] if s.get("args")), None))
    distinct = {frozenset(v) for v in by_cmd.values()}
    if len(distinct) > 1:
        raise AssertionError(
            "the header says `command` does not change what is asserted, but the commands carry "
            f"different assertion-key signatures: "
            f"{ {k: sorted(sorted(x) for x in v) for k, v in by_cmd.items()} }")
    return True


def assert_entries_declared(source, cases):
    """Every entry a step names on argv must be a declared `[source]` key.

    ADDED AFTER A POISON RUN THAT NOTHING CAUGHT. Renaming one `[source]` key
    without renaming the argv that selects it leaves a fail-closed case still
    green -- `kali build` fails on a MISSING FILE exactly as loudly as it fails
    on the program, so `exit = "failure"` cannot tell the two apart -- and it is
    invisible to every shipped gate as well: `check_fixtures.py` compares
    program TEXT, which is still present under the new key, and
    `audit-case-migration.py` sees no dropped literal. The case would silently
    stop exercising its program. U5 makes renames routine in this batch (four of
    the eleven files rename keys, the carrier seven times), so the
    correspondence is checked mechanically here rather than left to the fact
    that the generator happens to build both from one string.

    Also checks the two derived names U5 says must track a rename: a
    `browser_bundle_harness` `entry` and a `file_json` `path` are named after
    the build entry's STEM, because `kali build --bundle` names its output
    directory after the input stem.
    """
    for case in cases:
        entry_stem = None
        for step in case["steps"]:
            kind = step.get("kind", "cli")
            if kind == "cli":
                entry = step["args"][-1]
                if entry not in source:
                    raise AssertionError(
                        f"{case['name']}: argv names {entry!r}, which is not a `[source]` key")
                entry_stem = entry.rsplit(".", 1)[0]
                if entry_stem.endswith(".test"):
                    entry_stem = entry_stem[:-len(".test")]
            elif kind == "browser_bundle_harness":
                if entry_stem is None:
                    raise AssertionError(f"{case['name']}: harness step with no preceding build")
                if step["entry"] != entry_stem:
                    raise AssertionError(
                        f"{case['name']}: harness entry {step['entry']!r} does not track the "
                        f"build entry stem {entry_stem!r} (U5)")
            elif kind == "file_json":
                want = f"{entry_stem}/{entry_stem}.meta.json"
                if step["path"] != want:
                    raise AssertionError(
                        f"{case['name']}: file_json path {step['path']!r} does not track the "
                        f"build entry stem ({want!r} expected)")
    return True


def out(header, matrix, source, cases, *, non_axis=None):
    """Emit a case file, after every whole-file gate.

    `non_axis` is the PROVENANCE channel (ruling 18, step 2): the fail-closed
    non-axis block this file actually used, or None when it uses the stock §5.6
    wording. Threading it explicitly rather than recovering it from the rendered
    header is the whole point -- two rounds were lost to arms selected by
    matching prose that a reflow could silence.
    """
    assert_no_template_literals(source, cases)
    assert_entries_declared(source, cases)
    assert_non_axis_reason_matches(header, cases, non_axis)
    names = [c["name"] for c in cases]
    if len(set(names)) != len(names):
        dup = sorted({n for n in names if names.count(n) > 1})
        raise AssertionError(f"duplicate case name(s): {dup}")
    return emit_checked(header, matrix, source, cases)


def _gate_helper_sum(helpers, invocations):
    """`matrix_arithmetic`'s per-helper decomposition, with its sum GATED.

    `batch5_prose.matrix_arithmetic` checks `cases x axis == invocations`; it
    does not check that the per-helper counts printed beside it add up. Those
    are the figures a reader uses to follow the arithmetic, and nothing else
    reads them, so they are summed here and raise if they disagree with the
    total (ruling 15's first answer applied to a decomposition rather than to a
    single number).
    """
    total = sum(n for _name, n, _why in helpers)
    if total != invocations:
        raise AssertionError(
            f"per-helper invocation counts sum to {total}, not {invocations}")
    return helpers


def matrix_block(*, helpers, invocations, **kw):
    """`gen_batch7a.matrix_block`, with the helper decomposition gated first."""
    return _matrix_block(helpers=_gate_helper_sum(helpers, invocations),
                         invocations=invocations, **kw)


def matrix_arithmetic(*, helpers, invocations, **kw):
    """`batch5_prose.matrix_arithmetic`, with the helper decomposition gated."""
    return P.matrix_arithmetic(helpers=_gate_helper_sum(helpers, invocations),
                               invocations=invocations, **kw)


def arithmetic(label, *, fns, invocations, cases, axis_len):
    """Print the closing arithmetic and RAISE if it does not close (rule 7)."""
    if cases * axis_len != invocations:
        raise AssertionError(
            f"{label}: {cases} cases x {axis_len} = {cases * axis_len}, but the source makes "
            f"{invocations} invocations")
    print(f"  {label}: {fns} #[test] fns, {invocations} invocations, {cases} case(s) "
          f"x axis({axis_len}) = {invocations} trials -- closes")


# ==========================================================================
# T1. browser_object_keys_break_continue_harness.rs
#     16 fns / 16 invocations, [matrix] ext, fail-closed.
# ==========================================================================

@target("object_keys_break_continue_harness")
def gen_object_keys_break_continue_harness():
    stem = "object_keys_break_continue_harness"
    text = rs(stem)
    env = assert_env_name()
    helper = "assert_browser_harness_object_keys_break_continue"

    run_body = check_program("break/continue run", fixture_in_fn(
        text, "browser_harness_object_keys_break_continue_run_source"))
    test_body = check_program("break/continue test", fixture_in_fn(
        text, "browser_harness_object_keys_break_continue_test_source"))
    if run_body == test_body:
        raise AssertionError("the run and test fixtures are identical -- the split is pointless")
    source = {"main.${ext}": run_body, "smoke.test.${ext}": test_body}

    # The snippet deliberately starts at the `.expect`, not at `fs::write(`: U8's checker
    # takes the text before the first `(` of a backticked span as a cited identifier, and
    # `write` is not a fn in this source, so the obvious spelling turns the U8 arm red on
    # correct prose. Same class of collision as U5's `import()`/`require()` note.
    c_write = cite(text, '.expect("write source")')
    c_env = cite(text, "kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV")
    c_fail = cite(text, "assert!(!output.status.success()")

    blocks = blocks_in_fn(text, helper)
    non_axis = CLAIM_FREE_BLOCK
    header = hdr(
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        rule12_block(stem, blocks, reaching=UNIVERSAL_HELPER.format(helper=helper)),
        "",
        matrix_block(
            test_fns=16, invocations=16, cases=4, axis="ext", values=EXTS4,
            helpers=[(helper, 16,
                      "command(run/test) x ext(js/ts/jsx/tsx) x json_output(false/true), a "
                      "complete cross product. Every `#[test]` fn is one unlooped call and "
                      "the file contains no loop at all; both fixture builders are "
                      "parameterless, so `ext` really is uniform.")],
            non_axis_lines=non_axis),
        "",
        P.rule6_matrix_fold("4 source `#[test]` fns, one per `ext` cell"),
        "",
        P.u2_source_file_wide(list(source)),
        "",
        NO_TEMPLATE_LITERAL,
        "",
        P.rule13_header(["kali_bin",
                         "browser_harness_object_keys_break_continue_run_source",
                         "browser_harness_object_keys_break_continue_test_source", helper],
                        runner_exemption=False),
        "This file runs no `browser_bundle_harness` step, so its call chain never reaches",
        "`kali_runtime_contract`'s two harness helpers and ruling 6's exemption has nothing",
        "to exempt here; it is not stated.",
        "",
        ARGV_ORDER_HARNESS_ONLY,
        f"The env value is `{env}`, read from the",
        "`kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV` constant the source passes by",
        f"name {c_env} rather than assumed.",
        "",
        FAIL_CLOSED_NOTE,
        f"The helper writes whichever fixture its caller passed {c_write} and its only",
        f"assertion is at {c_fail}. There is no stdout, stderr, exit-code or JSON claim",
        "anywhere in the file, in either output mode, so the `--output json` siblings below",
        "assert nothing the text siblings do not. They are still their own `[[case]]` entries",
        "because the source has its own `#[test]` fns for them (rule 6) and because their",
        "argv genuinely differs.",
    )

    prose = prose_of(distinct_texts(blocks), stem)
    cases = []
    for command, entry in (("run", "main.${ext}"), ("test", "smoke.test.${ext}")):
        for json_output in (False, True):
            base = (("json_" if json_output else "") + command
                    + "_supports_object_keys_break_continue_when_browser_harness_is_configured")
            assert_fns(text, *[f"{base}_in_{e}_input" for e in EXTS4])
            cases.append({
                "name": base,
                "rationale": (
                    f"Migrated from browser_{stem}.rs, the four `{base}_in_*_input` fns (one "
                    f"per extension). `{helper}` writes the break/continue Object.keys "
                    f"{command} fixture {c_write} and asks `kali` to {command} it against the "
                    "browser API surface with the browser harness backed by node. The program "
                    "iterates Object.keys over a two-key object, continues past the first "
                    "key and breaks after pushing the second, then repeats the same loop "
                    "through a frozen single-quoted parenthesized-receiver property spelling "
                    "and checks both collected arrays. kali does not support that: the "
                    f"source's ONLY assertion is that the process fails {c_fail}, so this step "
                    "carries `exit = \"failure\"` and nothing else. Adding a diagnostic code or "
                    "a stdout claim the source never made would be a rule-2 invention, and "
                    "`exit = \"failure\"` is exactly as strong as the assertion it replaces."
                    + (" This sibling issues the `--output json` argv; the source asserts "
                       "nothing at all about the envelope, so neither does this case."
                       if json_output else "")
                    + prose),
                "steps": [harness_cli_step(command, entry, json_output,
                                           asserts={"exit": "failure"},
                                           json_claims=None)],
            })
    arithmetic(stem, fns=16, invocations=16, cases=len(cases), axis_len=4)
    return out(header, {"ext": EXTS4}, source, cases, non_axis=non_axis)


# ==========================================================================
# T2. browser_object_keys_entries_spread_bundle.rs
#     6 fns / 8 invocations, [matrix] ext, build succeeds, harness fails closed.
# ==========================================================================

@target("object_keys_entries_spread_bundle")
def gen_object_keys_entries_spread_bundle():
    stem = "object_keys_entries_spread_bundle"
    text = rs(stem)
    helper = "assert_browser_bundle_object_keys_entries_spread"

    program = check_program("keys/entries spread bundle", fixture_in_fn(
        text, "browser_bundle_object_keys_entries_spread_source"),
        must_contain="function browserObjectKeysEntriesSpread()")
    source = {"app.${ext}": program}
    harness_body = check_program(
        "harness body", fixture_starting(text, helper, "const mod = await import("),
        must_contain="await mod.browserObjectKeysEntriesSpread();")

    # Two lines carry this construct -- the build's `assert!(output.status.success(),` and the
    # harness's `assert!(!output.status.success(),` -- so the occurrence is picked explicitly
    # rather than defaulted, and `expect` raises if a third ever appears.
    c_build_ok = cite(text, "output.status.success(),", occurrence=1, expect=2)
    c_meta = cite(text, 'assert_eq!(metadata["apiSurface"], "browser")')
    c_errors = cite(text, 'assert!(envelope["errors"]')
    c_payload = cite(text, 'assert_eq!(envelope["exitCode"], 0)')
    c_fail = cite(text, "assert!(!output.status.success()")
    c_or = cite(text, 'stderr.contains("Uncaught Error") || stderr.contains("unreachable")')

    needles = ["Uncaught Error", "unreachable"]
    disjunction = ('`assert!(stderr.contains("Uncaught Error") || '
                   'stderr.contains("unreachable"), "stderr: {stderr}")`')
    stream, also = harness_or(stem, program, harness_body, needles)

    header = hdr(
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        rule12_block(stem, blocks_in_fn(text, helper),
                     reaching=UNIVERSAL_HELPER.format(helper=helper)),
        "",
        matrix_block(
            test_fns=6, invocations=8, cases=2, axis="ext", values=EXTS4,
            helpers=[(helper, 8,
                      "ext(js/ts/jsx/tsx) x json_output(false/true), a complete cross product. "
                      "Six `#[test]` fns cover it because two of them -- one per output mode "
                      "-- loop over `[\"app.jsx\", \"app.tsx\"]` rather than being written out "
                      "separately; the single fixture builder is parameterless, so `ext` is "
                      "uniform.")],
            non_axes=("json_output",), non_axis_lines=None),
        "",
        P.rule6_matrix_fold(
            "3 source `#[test]` fns -- one per ext cell, except that the jsx and tsx cells come "
            "from the two loop iterations of a single `..._in_jsx_tsx_input` fn"),
        "",
        P.u2_source_file_wide(list(source)),
        "",
        NO_TEMPLATE_LITERAL,
        "",
        P.rule13_header(["kali_bin", "browser_bundle_object_keys_entries_spread_source", helper],
                        runner_exemption=True),
        "",
        ARGV_ORDER_BUILD_ONLY,
        "",
        RUNNER_HARNESS_STEP,
        "",
        "ASSERTION SHAPE, mirrored from the source and nothing more.",
        f"  * `exit = \"success\"` on the build {c_build_ok}.",
        "  * In json mode, the envelope's schemaVersion/command/success/exitCode",
        f"    {c_payload} and an empty `errors` array {c_errors}. THERE IS NO",
        "    payload artifactKind/bundleFormat CLAIM HERE: this source reads the emitted",
        "    metadata file for those instead, and rule 2 forbids adding an envelope claim it",
        "    never made.",
        f"  * `app/app.meta.json`'s apiSurface/artifactKind {c_meta}, asserted in BOTH modes,",
        "    because the source reads that file outside the `if json_output` block.",
        f"  * The browser-bundle harness FAILS CLOSED {c_fail} -- `exit = \"failure\"`, not",
        "    `\"success\"`. This is the one file in this batch where the build succeeds and the",
        "    harness does not.",
        "",
        rule11_block(disjunction=disjunction, cite_text=c_or,
                     per_mode=[("the browser-bundle harness, both output modes", stream)],
                     needle=needles[0], also_true=also, needles=needles),
    )

    prose = prose_of(distinct_texts(blocks_in_fn(text, helper)), stem)
    cases = []
    for json_output in (False, True):
        base = ("json_" if json_output else "") + "build_emits_object_keys_entries_spread_semantics"
        assert_fns(text, f"{base}_in_js_input", f"{base}_in_ts_input", f"{base}_in_jsx_tsx_input")
        cases.append({
            "name": base,
            "rationale": (
                f"Migrated from browser_{stem}.rs, the three `{base}_in_*_input` fns -- one for "
                "js, one for ts, and one that loops `[\"app.jsx\", \"app.tsx\"]` and so supplies "
                f"the jsx and tsx cells of this matrix. `{helper}` writes the keys/entries "
                "spread fixture, builds it with `kali build --bundle --api browser`"
                + (" with `--output json`" if json_output else "")
                + ", asserts the emitted app/app.meta.json metadata, then runs the bundle glue "
                  "under the browser-bundle-harness contract backed by node. The program "
                  "spreads Object.keys and Object.entries over a frozen Object.fromEntries "
                  "result through ten and nine root spellings respectively -- dotted, mixed, "
                  "bracketed, single-quoted, and frozen parenthesized-receiver -- and checks "
                  f"every collected array. The build succeeds {c_build_ok} and the metadata "
                  f"pins apiSurface/artifactKind {c_meta}, but the emitted bundle does not run: "
                  f"the harness FAILS CLOSED {c_fail}."
                + rule11_rationale(disjunction, c_or, stream, needles[0], also)
                + (" This sibling additionally asserts the build JSON envelope -- "
                   f"schemaVersion/command/success/exitCode {c_payload} and an empty `errors` "
                   f"array {c_errors} -- rather than plain text; output shape is not a matrix "
                   "axis because it changes the assertion shape, so it is a separate case. Note "
                   "the source makes no payload artifactKind/bundleFormat claim on the "
                   "envelope, so none is asserted here (rule 2)."
                   if json_output else "")
                + prose),
            "steps": [
                build_step("app.${ext}", json_output, asserts={"exit": "success"},
                           json_claims={"schemaVersion": 1, "command": "build", "success": True,
                                        "exitCode": 0, "errors": []}),
                meta_step("app"),
                harness_step("app", harness_body,
                             {"exit": "failure", f"{stream}_contains": [needles[0]]}),
            ],
        })
    arithmetic(stem, fns=6, invocations=8, cases=len(cases), axis_len=4)
    return out(header, {"ext": EXTS4}, source, cases)


def harness_script(body):
    """The exact script the case runner writes for `entry = "app"`.

    NOT hand-simulated. `browser_bundle_harness_script(dir, false, body)` is a
    `format!` over `browser_bundle_harness_prelude(dir, false)` and `body`, and
    rule 8 forbids hand-applying a `format!`. So the PRELUDE is a capture taken
    from the real helper, and the composition rule -- script == prelude + body --
    is PROVED here against a second capture of a complete script rather than
    read off the helper's Rust source. A prelude that stopped naming this bundle
    directory, or a helper that stopped being a plain concatenation, fails the
    generator instead of silently answering a rule-11 question about a script
    the runner never runs.
    """
    prelude = C.CAP_HARNESS_PRELUDE_APP
    for needed in ("./app/app.js", "./app/app.wasm"):
        if needed not in prelude:
            raise AssertionError(
                f"the captured harness prelude no longer names {needed} -- it is stale")
    probe_body = ("const mod = await import(bundleJs.href);\n"
                  "await mod.browserObjectKeysEntriesSpread();\n")
    if prelude + probe_body != C.CAP_HARNESS_SCRIPT_APP_ENTRIES_SPREAD:
        raise AssertionError(
            "browser_bundle_harness_script is no longer prelude + body -- composing a script "
            "that way would be hand-simulating a `format!` (rule 8)")
    return prelude + body


def harness_or(label, program, harness_body, needles):
    """Resolve a rule-11 OR taken on the BROWSER-BUNDLE HARNESS's own streams.

    The harness is not a `kali` invocation, so `_stream`/`_needle` (which run
    `kali`) cannot answer it. The build is run for every extension, the harness
    script comes from `harness_script` above, and it is executed from the TRIAL
    ROOT -- which is where `steps.rs` runs it, and which differs from the
    source's cwd (the bundle dir). Both cwds are checked and must agree, so a
    claim that happens to hold only under the source's cwd cannot be pinned.
    """
    import shutil
    import subprocess
    import tempfile
    from kali_run import KALI
    if not os.path.exists(KALI):
        raise AssertionError(f"{KALI} absent -- {label}'s harness OR cannot be resolved")
    script = harness_script(harness_body)
    holds = {n: True for n in needles}
    streams = set()
    for ext in EXTS4:
        d = tempfile.mkdtemp()
        try:
            open(os.path.join(d, f"app.{ext}"), "w").write(program)
            p = subprocess.run([KALI, "build", "--bundle", "--api", "browser", f"app.{ext}"],
                               cwd=d, capture_output=True)
            if p.returncode != 0:
                raise AssertionError(f"{label}: build failed for app.{ext}: {p.stderr!r}")
            hp = os.path.join(d, "browser-bundle-smoke.mjs")
            open(hp, "w").write(script)
            for cwd in (d, os.path.join(d, "app")):
                q = subprocess.run(["node", hp], cwd=cwd, capture_output=True)
                seen = {"stdout": q.stdout.decode(), "stderr": q.stderr.decode()}
                for n in needles:
                    carriers = [s for s, t in seen.items() if n in t]
                    if len(carriers) > 1:
                        raise AssertionError(
                            f"{label} app.{ext}: {n!r} appears on both streams -- the OR does "
                            "not resolve to exactly one stream")
                    if not carriers:
                        holds[n] = False
                    else:
                        streams.add(carriers[0])
        finally:
            shutil.rmtree(d)
    if len(streams) != 1:
        raise AssertionError(f"{label}: cells disagree on the carrying stream: {streams}")
    universal = [n for n in needles if holds[n]]
    if not universal:
        raise AssertionError(f"{label}: no disjunct holds on every cell")
    if universal[0] != needles[0]:
        raise AssertionError(
            f"{label}: the first source-order disjunct {needles[0]!r} does not hold everywhere; "
            "the header's source-order tie-break would be describing something else")
    return streams.pop(), universal[1:]


# ==========================================================================
# T3. browser_object_keys_entries_spread_harness.rs
#     2 fns / 32 invocations, [matrix] ext, fail-closed, rule-8 captures.
# ==========================================================================

@target("object_keys_entries_spread_harness")
def gen_object_keys_entries_spread_harness():
    stem = "object_keys_entries_spread_harness"
    text = rs(stem)
    env = assert_env_name()
    helper = "assert_browser_harness_object_keys_entries_spread"
    builder = "browser_harness_object_keys_entries_spread_source"
    frozen_builder = "browser_harness_object_keys_entries_frozen_spread_source"

    run_plain = check_program("entries spread run", fixture_in_fn(text, builder, index=1))
    test_plain = check_program("entries spread test", fixture_in_fn(text, builder, index=0))
    _run, ws_run, lead_run, occ_run = assert_frozen_pair(
        "entries spread run", text, C.CAP_ENTRIES_SPREAD_RUN_PLAIN,
        C.CAP_ENTRIES_SPREAD_RUN_FROZEN, plain_literal=run_plain, replace_fn=frozen_builder)
    _test, ws_test, lead_test, occ_test = assert_frozen_pair(
        "entries spread test", text, C.CAP_ENTRIES_SPREAD_TEST_PLAIN,
        C.CAP_ENTRIES_SPREAD_TEST_FROZEN, plain_literal=test_plain, replace_fn=frozen_builder)
    ws_active = ws_run or ws_test
    lead, occ = P.assert_identical("needle indent", lead_run, lead_test), max(occ_run, occ_test)

    source = {
        "main.${ext}": C.CAP_ENTRIES_SPREAD_RUN_PLAIN,
        "main_frozen.${ext}": C.CAP_ENTRIES_SPREAD_RUN_FROZEN,
        "smoke.test.${ext}": C.CAP_ENTRIES_SPREAD_TEST_PLAIN,
        "smoke_frozen.test.${ext}": C.CAP_ENTRIES_SPREAD_TEST_FROZEN,
    }
    u5_check(source, ["main_frozen.${ext}", "smoke_frozen.test.${ext}"])

    c_fail = cite(text, "assert!(!output.status.success()")
    c_env = cite(text, "kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV")
    c_or = cite(text, 'stderr.contains("Uncaught Error")')
    c_replace = cite(text, f"    browser_harness_object_keys_entries_spread_source(test_mode)"
                           f".replace(")

    needles = ["Uncaught Error", "unreachable"]
    disjunction = ('`assert!(stderr.contains("Uncaught Error") || '
                   'stderr.contains("unreachable") || stdout.contains("Uncaught Error") || '
                   'stdout.contains("unreachable"), "stdout: {stdout}\\nstderr: {stderr}")`')
    streams, also = cli_or(stem, source, needles,
                           [("run", "main.${ext}"), ("test", "smoke.test.${ext}")])

    blocks = blocks_in_fn(text, helper)
    non_axis = fail_closed_non_axes_with_claim(text)
    header = hdr(
        extra_ok_block([(v, P.EXTRA_OK_U5_RENAME)
                        for v in expanded(["main_frozen.${ext}", "smoke_frozen.test.${ext}"])]),
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        rule12_block(stem, blocks, reaching=UNIVERSAL_HELPER.format(helper=helper)),
        "",
        matrix_block(
            test_fns=2, invocations=32, cases=8, axis="ext", values=EXTS4,
            helpers=[(helper, 32,
                      "command(run/test) x ext(js/ts/jsx/tsx) x fixture(plain/frozen) x "
                      "json_output(false/true), a complete cross product. Both `#[test]` fns "
                      "loop all four extensions and make four calls per extension, so `ext` is "
                      "uniform.")],
            non_axis_lines=non_axis),
        "",
        P.RULE6_ONE_TO_ONE,
        "That is the shape rule 5 governs rather than rule 6: this source has only TWO `#[test]`",
        "fns and each makes SIXTEEN independent invocations over four independent programs, so",
        "each is split into named siblings (by fixture and by output mode) rather than folded",
        "into one case. The names carry the loop coordinate, not a number.",
        "",
        P.u2_source_file_wide(list(source)),
        "",
        P.u5_renames([
            ("main.<ext>", "main_frozen.${ext}",
             "the frozen variant of the run program, which the source writes to the same "
             "`main.<ext>` name in a different test"),
            ("smoke.test.<ext>", "smoke_frozen.test.${ext}",
             "the frozen variant of the test program; the `.test.` infix is preserved because "
             "`kali test` selects its entry by that name shape"),
        ], collision="four different program texts to just two filenames"),
        "",
        NO_TEMPLATE_LITERAL,
        "",
        "RULE 8 / RULE 9 -- TWO OF THESE FOUR PROGRAMS ARE NOT STRING LITERALS.",
        f"{c_replace} builds the frozen variants by a str::replace off the plain ones, so there",
        "is no literal for the lexer to extract and every other route is hand-derivation --",
        "exactly the trap rule 8 exists to prevent. Both frozen texts are therefore the",
        "BYTE-EXACT OUTPUT of executing the real code (see `batch7b_captures.py` for the capture",
        "procedure).",
        wrap(f"The replace needle also carries {lead} LEADING SPACE(S) -- measured, and "
             "compared against the width the shipped captures were taken at, so a reindented "
             "source fails the generator rather than leaving this sentence wrong. Whether that "
             "indentation is load-bearing is MEASURED too, not asserted: "
             + ("it is -- the stripped needle selects a different span."
                if ws_active else
                f"it is not. The needle occurs {occ} time(s) in the plain text and the stripped "
                "form selects the same span, so the indentation hazard is LATENT here, not the "
                "operative reason. The check stays as a staleness tripwire."), 86),
        "The generator",
        "re-proves each capture against this source before emitting it: the plain capture must",
        "be byte-identical to the literal the lexer extracts from the `.rs`, the needle and",
        "replacement are read out of the `.rs` rather than restated, and applying them to the",
        "plain capture must reproduce the frozen capture exactly.",
        "",
        P.rule13_header(["kali_bin", builder, frozen_builder, helper], runner_exemption=False),
        "This file runs no `browser_bundle_harness` step, so its call chain never reaches",
        "`kali_runtime_contract`'s two harness helpers and ruling 6's exemption has nothing",
        "to exempt here; it is not stated.",
        "",
        ARGV_ORDER_HARNESS_ONLY,
        f"The env value is `{env}`, read from the",
        "`kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV` constant the source passes by",
        f"name {c_env} rather than assumed.",
        "",
        FAIL_CLOSED_NOTE,
        f"The source asserts the process fails {c_fail} and makes ONE further claim, which is",
        "OR-SHAPED.",
        "",
        rule11_block(disjunction=disjunction, cite_text=c_or,
                     per_mode=[("text mode", streams[False]),
                               ("`--output json`", streams[True])],
                     needle=needles[0], also_true=also, needles=needles),
    )

    prose = prose_of(distinct_texts(blocks), stem)
    plan = [
        ("run", "plain", "main.${ext}",
         "run_supports_object_keys_and_entries_spread_iteration_when_browser_harness_is_configured"),
        ("run", "frozen", "main_frozen.${ext}",
         "run_supports_object_keys_and_entries_spread_iteration_when_browser_harness_is_configured"),
        ("test", "plain", "smoke.test.${ext}",
         "test_supports_object_keys_and_entries_spread_iteration_when_browser_harness_is_configured"),
        ("test", "frozen", "smoke_frozen.test.${ext}",
         "test_supports_object_keys_and_entries_spread_iteration_when_browser_harness_is_configured"),
    ]
    assert_fns(text, *{fn for _c, _v, _e, fn in plan})
    cases = []
    for command, variant, entry, fn in plan:
        for json_output in (False, True):
            stream = streams[json_output]
            cases.append({
                "name": f"{fn}__{variant}"
                        + ("_json" if json_output else "_text"),
                "rationale": (
                    f"Migrated from browser_{stem}.rs, the source fn `{fn}`, its {variant} "
                    "fixture in "
                    + ("`--output json`" if json_output else "text")
                    + " mode (rule 5: that one fn makes sixteen independent invocations over "
                      "four independent programs, so it becomes named siblings carrying the "
                      f"loop coordinate). `{helper}` writes the program and asks `kali` to "
                    f"{command} it against the browser API surface with the browser harness "
                    "backed by node. The program spreads Object.keys and Object.entries over an "
                    "Object.fromEntries result through ten and nine root spellings and checks "
                    "every collected array"
                    + (", with the fromEntries result itself wrapped in `Object.freeze(...)`"
                       if variant == "frozen" else "")
                    + f". kali fails closed on it: the source asserts the process fails {c_fail} "
                      "and that a runtime diagnostic appears."
                    + rule11_rationale(disjunction, c_or, stream, needles[0], also)
                    + prose),
                "steps": [harness_cli_step(
                    command, entry, json_output, json_claims=None,
                    asserts={"exit": "failure", f"{stream}_contains": [needles[0]]})],
            })
    arithmetic(stem, fns=2, invocations=32, cases=len(cases), axis_len=4)
    return out(header, {"ext": EXTS4}, source, cases, non_axis=non_axis)


def cli_or(label, source, needles, cells):
    """Resolve a rule-11 OR taken on a `kali run|test` invocation's streams.

    Returns `({json_output: stream}, also_true)`. Every extension is run in both
    output modes; the stream must be unambiguous per mode and consistent across
    extensions, and the returned needle tie-break is the first source-order
    disjunct that holds on every cell.
    """
    per_mode = {}
    also = None
    for json_output in (False, True):
        probes = []
        for command, key in cells:
            body = source[key]
            for ext in EXTS4:
                entry = key.replace("${ext}", ext)
                argv = (["--output", "json"] if json_output else [])
                argv += [command, "--api", "browser", "--max-threads", "0",
                         "--max-spawned-processes", "0", entry]
                probes.append((entry, body, argv, needles[0], {HARNESS_ENV: "node"}))
        stream = _stream(f"{label} json={json_output}", probes)
        per_mode[json_output] = stream
        n, rest = _needle(f"{label} json={json_output}", stream, needles,
                          [(e, b, a, env) for e, b, a, _n, env in probes])
        if n != needles[0]:
            raise AssertionError(
                f"{label}: the first source-order disjunct {needles[0]!r} does not hold on "
                "every cell; the header's source-order tie-break would describe something else")
        also = rest if also is None else also
        if also != rest:
            raise AssertionError(f"{label}: the two output modes disagree on which disjuncts "
                                 "are additionally true")
    return per_mode, also


# ==========================================================================
# T4. browser_object_keys_integer_like_iteration.rs
#     12 fns / 26 invocations, MATRIX DECLINED (build covers js/ts only).
# ==========================================================================

@target("object_keys_integer_like_iteration")
def gen_object_keys_integer_like_iteration():
    stem = "object_keys_integer_like_iteration"
    text = rs(stem)
    env = assert_env_name()
    bundle_helper = "assert_integer_like_object_keys_iteration"
    harness_helper = "assert_browser_harness_integer_like_object_keys_iteration"

    bundle_body = check_program("integer-like bundle", fixture_in_fn(
        text, "integer_like_object_keys_iteration_source"),
        must_contain="function browserIntegerLikeObjectKeysIteration()")
    run_body = check_program("integer-like run", fixture_in_fn(
        text, "integer_like_object_keys_iteration_run_source"))
    test_body = check_program("integer-like test", fixture_in_fn(
        text, "integer_like_object_keys_iteration_test_source"))
    harness_body = check_program(
        "harness body", fixture_starting(text, bundle_helper, "const mod = await import("),
        must_contain="await mod.browserIntegerLikeObjectKeysIteration();")

    source = {}
    for ext in ("js", "ts"):
        source[f"app.{ext}"] = bundle_body
    for ext in EXTS4:
        source[f"main.{ext}"] = run_body
    for ext in EXTS4:
        source[f"smoke.test.{ext}"] = test_body
    # Ruling 7's MANDATORY half. This file declines `[matrix]`, so each extension is its own
    # literal `[source]` key and the three fixture groups really are duplicated across keys --
    # the one file in this batch where that happens. Ruling 7 declines U13's hoist for
    # `browser/` but makes the identity assertion mandatory: "duplication without a check is
    # just duplication".
    assert_bodies_identical("the bundle fixture, written by two cells", source,
                            [f"app.{e}" for e in ("js", "ts")])
    assert_bodies_identical("the run fixture, written by four cells", source,
                            [f"main.{e}" for e in EXTS4])
    assert_bodies_identical("the test fixture, written by four cells", source,
                            [f"smoke.test.{e}" for e in EXTS4])

    needle = "integer-like object enumeration ok"
    ok1 = "ok 1"
    pin = _pin("integer-like json.stdout", needle + "\n",
               [(f"main.{e}", run_body, "run") for e in EXTS4]
               + [(f"smoke.test.{e}", test_body, "test") for e in EXTS4])

    c_build_ok = cite(text, "output.status.success(),", occurrence=1, expect=3)
    c_errors = cite(text, 'envelope["errors"]')
    c_env_exit = cite(text, 'assert_eq!(envelope["exitCode"], 0)')
    c_harness_ok = cite(text, "output.status.success(),", occurrence=2, expect=3)
    # THREE lines carry this construct, in three different helpers, and the occurrence is
    # picked per case rather than reused: :1 is the bundle helper's `kali build`, :2 is the
    # bundle helper's browser-bundle harness run, :3 is the CLI harness helper's `kali
    # run|test`. A citation resolving to a construct in a DIFFERENT helper than the case it
    # annotates is reported CORRECT by the citation gate -- the construct really is on that
    # line -- so only picking the occurrence makes the citation mean what the prose says.
    c_cli_ok = cite(text, "output.status.success(),", occurrence=3, expect=3)
    # Snippet starts at the `(`, not at `String::from_utf8_lossy(`: U8 reads the text before a
    # backticked span's first `(` as a cited identifier, and `from_utf8_lossy` is not a fn in
    # this source, so the obvious spelling turns the U8 arm red on correct prose.
    c_empty = cite(text, "(&output.stdout).is_empty()")
    c_json_stdout = cite(text, 'let stdout = json["stdout"].as_str().expect("stdout string")')
    c_json_stderr = cite(text, 'assert_eq!(json["stderr"], "")')
    c_ok1 = cite(text, 'stdout.contains("ok 1")')
    c_env = cite(text, "kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV")
    c_no_meta = cite(text, 'let bundle_dir = dir.path().join("app")')

    header = hdr(
        extra_ok_block([(pin, P.EXTRA_OK_JSON_STDOUT)]),
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        rule12_none(stem, text),
        "",
        P.matrix_declined(
            test_fns=12, invocations=26, cases=26,
            reason=[
                "`ext` LOOKS like this family's usual four-value axis, and it is uniform for the",
                "EIGHT harness fns -- but it is not uniform file-wide. The bundle helper",
                f"`{bundle_helper}` is called for `app.js` and `app.ts` ONLY; there is no jsx or",
                "tsx bundle test in this source at all. An `ext` axis is FILE-WIDE with no",
                "per-case opt-out, so it would fan the two build cases into `app.jsx`/`app.tsx`",
                "combinations the source never ran -- a rule-2 invention.",
                "The harness half is not uniform either: three of its fns loop",
                "`[\"main.ts\", \"main.jsx\", \"main.tsx\"]` (or the `smoke.test.*` equivalent) while",
                "their `_in_js_input` siblings are separate unlooped fns, and two of those loops",
                "ALSO loop `json_output`, which no axis can express.",
            ]),
        "",
        P.RULE6_ONE_TO_ONE,
        "Four of the twelve fns are LOOPS -- over three filenames, and two of them over",
        "`json_output` as well -- each making several independent invocations. Rule 5 splits",
        "those into named siblings suffixed with the extension (and, where it varies, the output",
        "mode) they ran, not numbered, which is why 12 fns become 26 cases.",
        "",
        P.migration_note_stale_fn_name(
            "run_supports_integer_like_object_keys_iteration_when_browser_harness_is_configured"
            "_in_ts_jsx_tsx_input",
            "it carries no `json_` prefix, yet its body loops `json_output` over both values and "
            "so issues `--output json` for half of its six invocations -- the same shape as its "
            "`test_` sibling. Its `json_`-prefixed counterpart then repeats exactly those "
            "`--output json` invocations, so three of this source's 26 invocations are made "
            "twice, by two differently-named fns."),
        "",
        "DUPLICATE INVOCATIONS ARE PRESERVED, NOT COLLAPSED (rule 6). The three (`run`,",
        "`main.<ts|jsx|tsx>`, `--output json`) invocations are made by BOTH the unprefixed",
        "looping fn and its `json_`-prefixed counterpart, and likewise for `test`. Each fn keeps",
        "its own `[[case]]`, because the case is the only remaining trace of the fn; folding the",
        "pair would delete one fn from the record while leaving the trial count unchanged.",
        "",
        P.u2_source_file_wide(sorted(source)),
        "",
        P.RULING7_NO_HOIST,
        "",
        NO_TEMPLATE_LITERAL,
        "",
        P.rule13_header(["kali_bin", "integer_like_object_keys_iteration_source",
                         "integer_like_object_keys_iteration_run_source",
                         "integer_like_object_keys_iteration_test_source",
                         bundle_helper, harness_helper],
                        runner_exemption=True),
        "",
        "ARGV ORDER is transcribed in the exact order each source helper's `Command` builder",
        "appends it, which differs between the two helper shapes and is not normalised here:",
        "  * build:   `build --bundle --api browser [--output json] <entry>` -- the",
        "             `--output json` pair is appended AFTER the subcommand and its flags.",
        "  * run/test: `[--output json] <run|test> --api browser --max-threads 0",
        "             --max-spawned-processes 0 <entry>` -- appended BEFORE the subcommand.",
        "The source passes an absolute `dir.path().join(filename)` as the entry; the case runner",
        "passes the bare filename relative to the trial dir, matching every previously shipped",
        "`browser/` case file.",
        f"The env value is `{env}`, read from the",
        "`kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV` constant the source passes by",
        f"name {c_env} rather than assumed.",
        "",
        RUNNER_HARNESS_STEP,
        "",
        "ASSERTION SHAPE, mirrored from the source and nothing more.",
        f"  * BUILD: `exit = \"success\"` {c_build_ok}; in json mode the envelope's",
        f"    schemaVersion/command/success/exitCode {c_env_exit} and an empty `errors` array",
        f"    {c_errors}. THERE IS NO `file_json` STEP ON THESE CASES, and that omission is",
        "    deliberate: this source computes the bundle directory",
        f"    {c_no_meta} only to locate the harness script beside it and NEVER READS",
        "    `app/app.meta.json` -- unlike every other bundle target in this batch. Adding the",
        "    metadata assertion its siblings make would be a rule-2 invention.",
        f"  * BUILD, harness step: `exit = \"success\"` {c_harness_ok} and",
        f"    {c_empty}, which is an EXACT emptiness claim rather than a substring one, so it is",
        "    carried as `stdout = \"\"` -- the exact key, per ruling 3's mirror-the-source",
        "    direction for an already-exact assertion.",
        f"  * HARNESS, text mode: `stdout.contains(\"{needle}\")`, plus `\"{ok1}\"` {c_ok1} for the",
        "    `test` command only.",
        "  " + P.ruling3_substring(),
        f"  * HARNESS, json mode: the same claim against the string leaf json[\"stdout\"]",
        f"    {c_json_stdout}, plus schemaVersion/command/success, payload hostContract/",
        "    runtimeBackend, `exitCode = 0` at BOTH envelope and payload level for `run` or",
        f"    payload total/passed/failed for `test`, `json.stderr = \"\"` {c_json_stderr} and an",
        "    empty `errors` array.",
        "  " + P.ruling3_json_leaf(),
    )

    def bundle_case(name, entry, json_output, fn):
        return {
            "name": name,
            "rationale": (
                f"Migrated from browser_{stem}.rs, the source fn `{fn}`. `{bundle_helper}` writes "
                "the integer-like enumeration fixture, builds it with "
                "`kali build --bundle --api browser`"
                + (" with `--output json`" if json_output else "")
                + ", and runs the emitted bundle glue under the browser-bundle-harness contract "
                  "backed by node. The program collects Object.keys and Object.values over an "
                  "object whose keys mix integer-like names (10, 2, 1, 0) with string names (b, "
                  "a), and throws unless the integer-like keys come first in ascending numeric "
                  f"order followed by the string keys in insertion order. Both processes succeed "
                  f"({c_build_ok}, {c_harness_ok}) and the harness prints nothing at all "
                  f"{c_empty}, carried as the exact `stdout = \"\"`. This source never reads "
                  f"app/app.meta.json {c_no_meta}, so unlike its sibling bundle targets this case "
                  "asserts no metadata (rule 2)."
                + (" This sibling additionally asserts the build JSON envelope -- "
                   f"schemaVersion/command/success/exitCode {c_env_exit} and an empty `errors` "
                   f"array {c_errors} -- rather than plain text; output shape is not a matrix "
                   "axis because it changes the assertion shape, so it is a separate case."
                   if json_output else "")),
            "steps": [
                build_step(entry, json_output, asserts={"exit": "success"},
                           json_claims={"schemaVersion": 1, "command": "build", "success": True,
                                        "exitCode": 0, "errors": []}),
                harness_step("app", harness_body, {"exit": "success", "stdout": ""}),
            ],
        }

    def harness_case(name, command, entry, json_output, fn, note):
        asserts = {"exit": "success"}
        if json_output:
            claims = envelope_harness(command, stderr=True, errors=True)
            claims["stdout"] = pin
        else:
            claims = None
            asserts["stdout_contains"] = ([needle, ok1] if command == "test" else [needle])
        return {
            "name": name,
            "rationale": (
                f"Migrated from browser_{stem}.rs, the source fn `{fn}`{note}. "
                f"`{harness_helper}` writes the integer-like enumeration {command} fixture and "
                f"asks `kali` to {command} it against the browser API surface with the browser "
                "harness backed by node"
                + (", with `--output json`" if json_output else "")
                + ". The program collects Object.keys and Object.values over an object whose "
                  "keys mix integer-like names (10, 2, 1, 0) with string names (b, a), throws "
                  "unless the integer-like keys come first in ascending numeric order followed "
                  "by the string keys in insertion order, and prints one line on success. The "
                  f"process succeeds {c_cli_ok}"
                + (f", and the source takes its output claim against the string leaf "
                   f"json[\"stdout\"] {c_json_stdout}, also pinning "
                   + ("`exitCode = 0` at both envelope and payload level"
                      if command == "run" else "the payload's total, passed and failed")
                   + f", `json.stderr = \"\"` {c_json_stderr} and an empty `errors` array. "
                   + P.ruling3_json_leaf()
                   if json_output else
                   f", and the output claim is a plain `.contains(\"{needle}\")` against raw "
                   f"stdout"
                   + (f", plus a second needle {c_ok1} that only the `test` command emits"
                      if command == "test" else "")
                   + ". " + P.ruling3_substring())),
            "steps": [harness_cli_step(command, entry, json_output,
                                       asserts=asserts, json_claims=claims)],
        }

    cases = []
    for json_output in (False, True):
        for ext in ("js", "ts"):
            fn = (("json_" if json_output else "")
                  + f"build_emits_integer_like_object_keys_iteration_semantics_in_{ext}_input")
            assert_fns(text, fn)
            cases.append(bundle_case(fn, f"app.{ext}", json_output, fn))

    for command, stem_name in (("run", "main"), ("test", "smoke.test")):
        js_fn = (f"{command}_supports_integer_like_object_keys_iteration_when_browser_harness"
                 f"_is_configured_in_js_input")
        loop_fn = (f"{command}_supports_integer_like_object_keys_iteration_when_browser_harness"
                   f"_is_configured_in_ts_jsx_tsx_input")
        json_js_fn = "json_" + js_fn
        json_loop_fn = "json_" + loop_fn
        assert_fns(text, js_fn, loop_fn, json_js_fn, json_loop_fn)
        cases.append(harness_case(js_fn, command, f"{stem_name}.js", False, js_fn, ""))
        for ext in ("ts", "jsx", "tsx"):
            for json_output in (False, True):
                cases.append(harness_case(
                    f"{loop_fn}__{ext}" + ("_json" if json_output else "_text"),
                    command, f"{stem_name}.{ext}", json_output, loop_fn,
                    f", the {ext} / {'json' if json_output else 'text'} cell of its "
                    "three-filename x two-output-mode loop (rule 5: independent invocations "
                    "become named siblings carrying the loop coordinate, never one folded case)"))
        cases.append(harness_case(json_js_fn, command, f"{stem_name}.js", True, json_js_fn, ""))
        for ext in ("ts", "jsx", "tsx"):
            cases.append(harness_case(
                f"{json_loop_fn}__{ext}", command, f"{stem_name}.{ext}", True, json_loop_fn,
                f", the {ext} cell of its three-filename loop (rule 5). This invocation is "
                "IDENTICAL to one made by the unprefixed looping fn above; both are kept, "
                "because a `[[case]]` is the only remaining trace of the fn that made it "
                "(rule 6)"))

    if len(cases) != 26:
        raise AssertionError(f"expected 26 cases, built {len(cases)}")
    arithmetic(stem, fns=12, invocations=26, cases=len(cases), axis_len=1)
    return out(header, None, source, cases)


# ==========================================================================
# T5. browser_object_keys_iteration.rs -- THE #[path] SUBMODULE CARRIER.
#     0 top-level fns; 15 in build.rs and 10 in build_json.rs; 52 invocations.
# ==========================================================================

CARRIER_PROGRAMS = [
    # (renamed [source] stem, fixture builder in the carrier, exported JS fn)
    ("object_keys_iteration", "browser_bundle_object_keys_iteration_source",
     "browserObjectKeysIteration"),
    ("direct_object_keys_iteration", "browser_bundle_direct_object_keys_iteration_source",
     "browserDirectObjectKeysIteration"),
    ("global_object_keys_iteration", "browser_bundle_global_object_keys_iteration_source",
     "browserGlobalObjectKeysIteration"),
    ("await_wrapped_static_object_helpers",
     "browser_bundle_await_wrapped_static_object_helpers_source",
     "browserAwaitWrappedStaticObjectHelpers"),
    ("const_bound_object_keys_iteration",
     "browser_bundle_const_bound_object_keys_iteration_source",
     "browserConstBoundObjectKeysIteration"),
    ("object_values_iteration", "browser_bundle_object_values_iteration_source",
     "browserObjectValuesIteration"),
    ("object_keys_break_continue_iteration",
     "browser_bundle_object_keys_break_continue_iteration_source",
     "browserObjectKeysBreakContinueIteration"),
]


@target("object_keys_iteration")
def gen_object_keys_iteration():
    stem = "object_keys_iteration"
    text = rs(stem)
    subs = submods(stem)
    if sorted(subs) != ["build.rs", "build_json.rs"]:
        raise AssertionError(f"unexpected submodule set: {sorted(subs)}")

    top_level = len(_local_test_fn_names(text))
    if top_level != 0:
        raise AssertionError(
            f"the carrier now has {top_level} top-level `#[test]` fns; U10's inventory and this "
            "generator's arithmetic both assume 0")
    counts = {name: len(_local_test_fn_names(t)) for name, t in subs.items()}
    if counts != {"build.rs": 15, "build_json.rs": 10}:
        raise AssertionError(f"submodule `#[test]` inventory moved: {counts}")

    bodies = {s: check_program(s, fixture_in_fn(text, fn), must_contain=f"function {js}")
              for s, fn, js in CARRIER_PROGRAMS}
    source = {f"{s}.${{ext}}": bodies[s] for s, _fn, _js in CARRIER_PROGRAMS}
    u5_check(source, list(source))

    bc_stem = "object_keys_break_continue_iteration"
    harness_body = check_program(
        "harness body",
        fixture_starting(subs["build.rs"],
                         "build_emits_object_keys_break_continue_iteration_semantics"
                         "_in_js_ts_jsx_tsx_input",
                         "const mod = await import("),
        must_contain="await mod.browserObjectKeysBreakContinueIteration();")

    c_fail = cite(text, "assert!(!output.status.success()", occurrence=1, expect=3)
    BC_FN = "build_emits_object_keys_break_continue_iteration_semantics_in_js_ts_jsx_tsx_input"
    c_bc_ok = sub_cite("build.rs", subs["build.rs"], "output.status.success(),",
                       occurrence=1, expect=2, within=BC_FN)
    c_bc_harness = sub_cite("build.rs", subs["build.rs"], "output.status.success(),",
                            occurrence=2, expect=2, within=BC_FN)
    c_bc_meta = sub_cite("build.rs", subs["build.rs"],
                         'assert_eq!(metadata["apiSurface"], "browser")', within=BC_FN)
    c_bc_errors = sub_cite("build.rs", subs["build.rs"], 'assert!(envelope["errors"]',
                           within=BC_FN)
    c_bc_exit = sub_cite("build.rs", subs["build.rs"], 'assert_eq!(envelope["exitCode"], 0)',
                         within=BC_FN)
    c_bc_empty = sub_cite("build.rs", subs["build.rs"],
                          "(&output.stdout).is_empty()", within=BC_FN)
    c_mods = cite(text, '#[path = "browser_object_keys_iteration/build.rs"]')

    blocks = [b for t in [text] + [subs[k] for k in sorted(subs)]
              for b in comment_blocks(t)]
    texts = distinct_texts(blocks)
    if len(texts) != 1:
        raise AssertionError(
            f"the carrier's prose is no longer one distinct text ({len(texts)}); the U6 "
            "attribution below names exactly one block and would be describing something else")


    prose = prose_of(texts, stem)

    def failing(name, program_stem, json_output, fns, describe, fail_cite):
        return {
            "name": name,
            "rationale": (
                f"Migrated from browser_{stem}.rs's `#[path]` submodules, {fns}. The case writes "
                f"the {program_stem.replace('_', ' ')} program to {program_stem}.<ext> and runs "
                "`kali build --bundle --api browser`"
                + (" with `--output json`" if json_output else "")
                + f". {describe} kali does not support that: the source's ONLY assertion is that "
                  f"the process fails {fail_cite}, so this step carries `exit = \"failure\"` and "
                  "nothing else. Adding a diagnostic code or a stdout claim the source never made "
                  "would be a rule-2 invention, and `exit = \"failure\"` is exactly as strong as "
                  "the assertion it replaces."
                + (" This sibling issues the `--output json` argv; the source asserts nothing at "
                   "all about the envelope, so neither does this case." if json_output else "")
                + prose),
            "steps": [build_step(f"{program_stem}.${{ext}}", json_output,
                                 asserts={"exit": "failure"}, json_claims=None)],
        }

    def sub_fns(sub, names):
        """Name the submodule `#[test]` fns a case was migrated from, WITHOUT a `:N`.

        A citation is exempt from ruling 11 ONLY because it is mechanically
        gated, and `batch5_crosscheck.py`'s resolver derives the needle it
        re-searches for from a `.method(` or a string literal inside the
        snippet. `fn <name>(` has neither, so a citation onto a bare fn
        signature MATCHES BUT SEARCHES FOR NOTHING -- it reports clean whether
        it is right or wrong, which is the "figure in disguise" ruling 11
        forbids. `citation_sweep.sh` records the disposition for that shape as
        REWORD rather than red-list, so the fn names are named plainly (U8 still
        resolves them against the submodules) and every `:N` in this file points
        at an assertion construct, which does carry a needle.
        """
        return ", ".join(f"`{n}`" for n in names)

    describe = {
        "object_keys_iteration":
            "The program iterates Object.keys over a two-key object through nine bracketed, "
            "single-quoted, mixed and frozen parenthesized-receiver root spellings and checks "
            "every collected array against the insertion order.",
        "direct_object_keys_iteration":
            "The program iterates Object.keys over an object literal passed directly, with no "
            "intervening binding.",
        "global_object_keys_iteration":
            "The program iterates Object.keys through eleven `globalThis`-rooted spellings -- "
            "dotted, mixed, fully bracketed, single-quoted and frozen parenthesized-receiver.",
        "await_wrapped_static_object_helpers":
            "The program wraps the operands of Object.hasOwn, Object.keys and Reflect.ownKeys in "
            "await expressions and in comma-sequence expressions, and checks each result.",
        "const_bound_object_keys_iteration":
            "The program iterates Object.keys over a const-bound object literal.",
        "object_values_iteration":
            "The program iterates Object.values over a two-key object reached through an alias "
            "binding.",
    }

    # THE FAIL ASSERT IS CITED PER CASE, SCOPED TO THE HELPER OR `#[test]` FN THAT MAKES IT.
    # The carrier spells `assert!(!output.status.success(), ...)` in three different helpers and
    # the two submodules spell it in seven more `#[test]` fns; a citation resolving into a
    # DIFFERENT helper than the case it annotates is reported CORRECT by the citation gate --
    # the construct really is on that line -- so the occurrence is picked rather than defaulted.
    FAILC = "assert!(!output.status.success()"

    def carrier_fail(occurrence):
        return cite(text, FAILC, occurrence=occurrence, expect=3)

    def sub_fail(sub, fns):
        return " and ".join(sub_cite(sub, subs[sub], FAILC, within=f) for f in fns)

    cases = []
    plan = [
        ("object_keys_iteration", "build.rs", "build_json.rs",
         [f"build_emits_object_keys_iteration_semantics_in_{e}_input" for e in EXTS4],
         [f"json_build_emits_object_keys_iteration_semantics_in_{e}_input" for e in EXTS4]),
        ("direct_object_keys_iteration", "build.rs", "build_json.rs",
         [f"build_emits_direct_object_keys_iteration_semantics_in_{e}_input" for e in EXTS4],
         [f"json_build_emits_direct_object_keys_iteration_semantics_in_{e}_input"
          for e in EXTS4]),
        ("global_object_keys_iteration", "build.rs", "build_json.rs",
         ["build_emits_global_object_keys_iteration_semantics_in_js_input",
          "build_emits_global_object_keys_iteration_semantics_in_ts_jsx_tsx_input"],
         ["json_build_emits_global_object_keys_iteration_semantics_in_js_input",
          "json_build_emits_global_object_keys_iteration_semantics_in_ts_jsx_tsx_input"]),
        ("const_bound_object_keys_iteration", "build.rs", None,
         ["build_emits_const_bound_object_keys_iteration_semantics_in_js_ts_jsx_tsx_input"],
         ["build_emits_const_bound_object_keys_iteration_semantics_in_js_ts_jsx_tsx_input"]),
        ("await_wrapped_static_object_helpers", "build.rs", None,
         ["build_emits_await_wrapped_static_object_helpers_in_js_ts_jsx_tsx_input"],
         ["build_emits_await_wrapped_static_object_helpers_in_js_ts_jsx_tsx_input"]),
        ("object_values_iteration", "build.rs", None,
         ["build_emits_object_values_iteration_semantics_in_js_input",
          "build_emits_object_values_iteration_semantics_in_ts_jsx_tsx_input"],
         None),
    ]
    # program stem -> (text-mode fail citation, json-mode fail citation). The three
    # helper-routed programs cite the CARRIER helper that makes the assert (one citation covers
    # both modes, because both modes go through the same helper); the four inlined programs cite
    # the `#[test]` fn(s) in the submodule that inline it.
    fail_cites = {
        "object_keys_iteration": (carrier_fail(1), carrier_fail(1)),
        "direct_object_keys_iteration": (carrier_fail(2), carrier_fail(2)),
        "await_wrapped_static_object_helpers": (carrier_fail(3), carrier_fail(3)),
        "const_bound_object_keys_iteration": (
            sub_fail("build.rs",
                     ["build_emits_const_bound_object_keys_iteration_semantics"
                      "_in_js_ts_jsx_tsx_input"]),) * 2,
        "global_object_keys_iteration": (
            sub_fail("build.rs",
                     ["build_emits_global_object_keys_iteration_semantics_in_js_input",
                      "build_emits_global_object_keys_iteration_semantics_in_ts_jsx_tsx_input"]),
            sub_fail("build_json.rs",
                     ["json_build_emits_global_object_keys_iteration_semantics_in_js_input",
                      "json_build_emits_global_object_keys_iteration_semantics"
                      "_in_ts_jsx_tsx_input"])),
        "object_values_iteration": (
            sub_fail("build.rs",
                     ["build_emits_object_values_iteration_semantics_in_js_input",
                      "build_emits_object_values_iteration_semantics_in_ts_jsx_tsx_input"]), None),
    }
    for program_stem, text_sub, json_sub, text_fn_names, json_fn_names in plan:
        for names in (text_fn_names, json_fn_names):
            if names:
                assert_fns(subs[json_sub or text_sub] if names is json_fn_names and json_sub
                           else subs[text_sub], *names)
        text_cite, json_cite = fail_cites[program_stem]
        cases.append(failing(
            f"build_emits_{program_stem}", program_stem, False,
            "the " + sub_fns(text_sub, text_fn_names) + " fn(s)", describe[program_stem],
            text_cite))
        if json_fn_names:
            sub = json_sub or text_sub
            label = ("the " + sub_fns(sub, json_fn_names) + " fn(s)"
                     if json_sub else
                     "the `--output json` half of the " + sub_fns(sub, json_fn_names)
                     + " loop (rule 5: its `for json_output in [false, true]` loop makes two "
                       "independent invocations per extension, so it becomes two named "
                       "siblings)")
            cases.append(failing(f"json_build_emits_{program_stem}", program_stem, True,
                                 label, describe[program_stem], json_cite))

    bc_fn = "build_emits_object_keys_break_continue_iteration_semantics_in_js_ts_jsx_tsx_input"
    assert_fns(subs["build.rs"], bc_fn)
    for json_output in (False, True):
        cases.append({
            "name": ("json_" if json_output else "") + "build_emits_"
                    + "object_keys_break_continue_iteration",
            "rationale": (
                f"Migrated from browser_{stem}.rs's `#[path]` submodule build.rs, the "
                f"{'`--output json`' if json_output else 'text'} half of `{bc_fn}` "
                "-- rule 5: that fn's `for json_output in [false, true]` loop makes two "
                "independent invocations per extension, so it becomes two named siblings. The "
                f"case writes the break/continue program to {bc_stem}.<ext>, builds it with "
                "`kali build --bundle --api browser`"
                + (" with `--output json`" if json_output else "")
                + ", asserts the emitted metadata, then runs the bundle glue under the "
                  "browser-bundle-harness contract backed by node. The program iterates "
                  "Object.keys over a two-key object, continues past the first key and breaks "
                  "after pushing the second, and throws unless exactly the second key was "
                  f"collected. This is the one program in this target kali DOES support: the "
                  f"build succeeds {c_bc_ok}, the metadata pins apiSurface/artifactKind "
                  f"{c_bc_meta}, the harness succeeds {c_bc_harness}, and the harness prints "
                  f"nothing at all {c_bc_empty} -- an exact emptiness claim, so it is carried as "
                  "the exact `stdout = \"\"` key rather than as an absence needle. The `[source]` "
                  f"key and therefore the bundle directory are {bc_stem} rather than the "
                  "source's `app` (U5): `kali build --bundle` names its output directory after "
                  "the input stem, so the `file_json` path and the harness `entry` track the "
                  "rename."
                + (f" This sibling additionally asserts the build JSON envelope -- "
                   f"schemaVersion/command/success/exitCode {c_bc_exit} and an empty `errors` "
                   f"array {c_bc_errors} -- rather than plain text; output shape is not a matrix "
                   "axis because it changes the assertion shape, so it is a separate case."
                   if json_output else "")),
            "steps": [
                build_step(f"{bc_stem}.${{ext}}", json_output, asserts={"exit": "success"},
                           json_claims={"schemaVersion": 1, "command": "build", "success": True,
                                        "exitCode": 0, "errors": []}),
                meta_step(bc_stem),
                harness_step(bc_stem, harness_body, {"exit": "success", "stdout": ""}),
            ],
        })

    header = hdr(
        extra_ok_block(
            [(v, P.EXTRA_OK_U5_RENAME) for v in expanded(list(source))]
            + [("", "the EXACT-EMPTINESS claim mirrored from the source's own "
                    "`(&output.stdout).is_empty()` on the break/continue harness. "
                    "`check_extra_claims.py` accepts an extra claim that appears verbatim in "
                    "the `.rs`, but guards that arm with `e and ...`, so the empty string can "
                    "never take it and must be declared instead. Its sibling "
                    "object_keys_integer_like_iteration.toml needs no such declaration only "
                    "because THAT source also spells `assert_eq!(json[\"stderr\"], \"\")`, "
                    "which puts the empty string among the audit's extracted claims; this "
                    "source makes no stderr claim at all")]),
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        "U10 -- THIS SOURCE IS A `#[path]` SUBMODULE CARRIER, AND ITS INVENTORY IS THE WHOLE",
        "POINT OF THE RULE. The carrier declares two submodules",
        f"({c_mods} and its `build_json.rs` sibling) and has ZERO top-level `#[test]` fns, so",
        "`grep -c '#[test]'` on the carrier returns 0 and silently drops every test in the",
        "target. Inventory, re-derived by this generator on every run from",
        "`audit-case-migration.py`'s own `resolve_path_mods` rather than by a second",
        "implementation of the same predicate:",
        "  * browser_object_keys_iteration.rs             0 `#[test]` fns",
        "  * browser_object_keys_iteration/build.rs       15 `#[test]` fns",
        "  * browser_object_keys_iteration/build_json.rs  10 `#[test]` fns",
        "The generator RAISES if any of those three counts moves, and the invocation arithmetic",
        "below closes against their total. Carrier and directory are deleted",
        "together, by the family-wide deletion after batch 8 -- not by this batch.",
        "",
        "WHY ONE CASE FILE AND NOT TWO, ONE PER SUBMODULE. Batch 6B split its carrier into two",
        "case files, so the question was derived here rather than inherited. U2 forces a split",
        "when two groups of tests need DIFFERENT file-wide `[source]` tables -- specifically",
        "when a fixture is written conditionally, or when a case's whole point is the PRESENCE",
        "or ABSENCE of a file (6B's `kali.json` manifest: merging would have made it",
        "unconditionally present and silently stopped its explicit half from discriminating).",
        "Neither condition holds here, and all three were checked rather than assumed:",
        "  * NO CONDITIONAL WRITE. Every fixture write in the carrier and in both submodules is",
        "    unconditional within its test; the generator finds no `if` guarding one.",
        "  * NO PRESENCE/ABSENCE CASE. Every `[source]` entry is a PROGRAM, named explicitly on",
        "    argv by the case that uses it. There is no manifest, config, lockfile or sibling",
        "    module anywhere in this target -- nothing whose mere presence changes another",
        "    case's behaviour.",
        "  * MEASURED, not argued: every one of the 7 programs was built by the real `kali` in",
        "    both output modes and all four extensions, alone in a directory and again with all",
        "    28 renamed fixtures co-resident, comparing exit code, stdout, stderr AND the",
        "    emitted bundle directory listing. 56 cells, 0 differences. The comparison was",
        "    proved able to report one by a negative control (the same diff with a different",
        "    entry filename), which does report a difference.",
        "So the two submodules can share one `[source]` table, and splitting would buy nothing",
        "while costing the joint audit its single-file left-hand side. One `.toml` is not a",
        "goal in itself; here it is simply the correct split.",
        "",
        rule12_block(stem, blocks, extra_files=sorted(subs), reaching=(
            "Every block is the same fail-closed re-pin note, and it sits in the three carrier "
            "assert helpers and in every `#[test]` fn that inlines its own `Command` and "
            "expects failure. It does NOT reach the break/continue tests, which carry no "
            "comment at "
            "all, so it is carried into the fail-closed rationales only -- see the U6 paragraph "
            "below, whose reach figures are counted from the emitted rationales.")),
        "",
        u6_partial_note([
            ("the three carrier assert helpers and the fail-closed `#[test]` fns that inline "
             "their own `Command`", texts[0]),
        ], cases),
        "",
        matrix_arithmetic(
            test_fns=25, invocations=52, cases=13, axis="ext", values=EXTS4,
            helpers=[
                ("assert_browser_bundle_object_keys_iteration", 8,
                 "ext(js/ts/jsx/tsx) x json_output(false/true), eight unlooped `#[test]` fns "
                 "split four-and-four across the two submodules"),
                ("assert_browser_bundle_direct_object_keys_iteration", 8,
                 "the same eight-fn shape for the direct-literal program"),
                ("assert_browser_bundle_await_wrapped_static_object_helpers", 8,
                 "one `#[test]` fn looping ext(4) x json_output(2)"),
                # The last four rows name the FIXTURE BUILDER rather than an assert helper,
                # because those programs have none: their `#[test]` fns inline their own
                # `Command`. The column still holds a real identifier U8 can resolve, which a
                # backticked phrase would not.
                ("browser_bundle_const_bound_object_keys_iteration_source", 8,
                 "one `#[test]` fn, inlining its own `Command`, looping ext(4) x "
                 "json_output(2)"),
                ("browser_bundle_global_object_keys_iteration_source", 8,
                 "four `#[test]` fns inlining their own `Command` -- a js one and a ts/jsx/tsx "
                 "loop, in each submodule"),
                ("browser_bundle_object_values_iteration_source", 4,
                 "two `#[test]` fns in build.rs inlining their own `Command` -- a js one and a "
                 "ts/jsx/tsx loop -- and NO json counterpart, which is the one program this "
                 "source never builds with `--output json`"),
                ("browser_bundle_object_keys_break_continue_iteration_source", 8,
                 "one `#[test]` fn, inlining its own `Command`, looping ext(4) x "
                 "json_output(2) -- the only one whose build SUCCEEDS"),
            ],
            non_axes=("json_output",)),
        "What makes the file-wide axis safe is that every one of the seven programs covers all",
        "four extensions uniformly, in each output mode it is exercised in at all -- so no case",
        "is fanned into a combination the source never ran (rule 2).",
        "",
        P.rule6_matrix_fold(
            "between one and four source `#[test]` fns -- four unlooped ones for the two "
            "eight-fn programs, two (a js fn plus a ts/jsx/tsx loop) for the global-root and "
            "object-values programs, and one looping fn for the rest"),
        "",
        P.u2_source_file_wide(sorted(source)),
        "",
        P.u5_renames(
            [("app.<ext>", f"{s}.${{ext}}", f"the {s.replace('_', ' ')} program")
             for s, _fn, _js in CARRIER_PROGRAMS],
            collision="seven different program texts to the one filename `app.<ext>`"),
        "The rename reaches further than argv for the break/continue case: `kali build --bundle`",
        "names its output directory after the input STEM, so that case's `file_json` path and",
        f"its `browser_bundle_harness` entry are {bc_stem} rather than the source's `app`, as",
        "U5 requires. That was verified against the real binary, not assumed.",
        "",
        NO_TEMPLATE_LITERAL,
        "",
        P.rule13_header(["kali_bin"] + [fn for _s, fn, _js in CARRIER_PROGRAMS]
                        + ["assert_browser_bundle_object_keys_iteration",
                           "assert_browser_bundle_direct_object_keys_iteration",
                           "assert_browser_bundle_await_wrapped_static_object_helpers"],
                        runner_exemption=True),
        "",
        ARGV_ORDER_BUILD_ONLY,
        "",
        RUNNER_HARNESS_STEP,
        "",
        FAIL_CLOSED_NOTE,
        f"{sum(1 for c in cases if c['steps'][0].get('exit') == 'failure')} of the "
        f"{len(cases)} cases assert nothing but process failure (e.g. {c_fail}, and each "
        "rationale below cites the helper or `#[test]` fn that makes ITS own assert): no stdout,",
        "stderr, exit-code or JSON claim is made anywhere in the carrier's three assert helpers",
        "or in the fail-closed `#[test]` fns that inline their own `Command`, in either output",
        "mode. The",
        "`--output json` siblings therefore assert nothing their text siblings do not; they are",
        "still their own `[[case]]` entries because the source has its own `#[test]` fns for",
        "them (rule 6) and because their argv genuinely differs.",
        "",
        "THE BREAK/CONTINUE PAIR IS THE EXCEPTION, and it is the only case in this file that",
        "asserts anything positive:",
        f"  * the build SUCCEEDS {c_bc_ok}, and in json mode pins the envelope's",
        f"    schemaVersion/command/success/exitCode {c_bc_exit} plus an empty `errors` array",
        f"    {c_bc_errors};",
        f"  * the emitted metadata's apiSurface/artifactKind {c_bc_meta}, asserted in BOTH modes",
        "    because the source reads that file outside the `if json_output` block;",
        f"  * the browser-bundle harness succeeds {c_bc_harness} and prints nothing at all",
        f"    {c_bc_empty} -- an EXACT emptiness claim, carried as the exact `stdout = \"\"` key",
        "    per ruling 3's mirror-the-source direction for an already-exact assertion.",
    )

    if len(cases) != 13:
        raise AssertionError(f"expected 13 cases, built {len(cases)}")
    arithmetic(stem, fns=25, invocations=52, cases=len(cases), axis_len=4)
    return out(header, {"ext": EXTS4}, source, cases)


# ==========================================================================
# T6. browser_object_string_enumeration_bundle.rs
#     16 fns / 16 invocations, [matrix] ext, fail-closed, two programs.
# ==========================================================================

@target("object_string_enumeration_bundle")
def gen_object_string_enumeration_bundle():
    stem = "object_string_enumeration_bundle"
    text = rs(stem)
    helper = "assert_browser_bundle_object_string_enumeration"
    await_helper = "assert_browser_bundle_object_string_enumeration_await"

    plain = check_program("string enumeration", fixture_in_fn(
        text, "browser_bundle_object_string_enumeration_source"),
        must_contain="function browserObjectStringEnumeration()")
    await_body = check_program("string enumeration await", fixture_in_fn(
        text, "browser_bundle_object_string_enumeration_await_source"),
        must_contain="export async function browserObjectStringEnumerationAwait()")
    if plain == await_body:
        raise AssertionError("the plain and await fixtures are identical -- the split is pointless")
    source = {"app.${ext}": plain, "app_await.${ext}": await_body}
    u5_check(source, ["app_await.${ext}"])

    c_fail = cite(text, "assert!(!output.status.success()", occurrence=1, expect=2)
    c_fail_by_helper = {h: cite_in(text, h, "assert!(!output.status.success()")
                        for h in (helper, await_helper)}

    non_axis = CLAIM_FREE_BUILD_BLOCK
    header = hdr(
        extra_ok_block([(v, P.EXTRA_OK_U5_RENAME) for v in expanded(["app_await.${ext}"])]),
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        rule12_block(stem, comment_blocks(text), reaching=(
            "Both blocks are the same fail-closed re-pin note and they sit in the file's two "
            "assert helpers, `" + helper + "` and `" + await_helper + "`, which between them "
            "produce every `[[case]]` below -- so U6's bottom-up attribution puts the text in "
            "every rationale. That is not the over-attribution U6 forbids (copying a block into "
            "cases its producing helper does not reach); here the two helpers partition the "
            "file and carry identical prose.")),
        "",
        matrix_block(
            test_fns=16, invocations=16, cases=4, axis="ext", values=EXTS4,
            helpers=[(helper, 8, "ext(js/ts/jsx/tsx) x json_output(false/true), eight unlooped "
                                 "`#[test]` fns"),
                     (await_helper, 8, "the same eight-fn shape for the `for await` variant")],
            non_axes=("json_output",), non_axis_lines=non_axis),
        "",
        P.rule6_matrix_fold("4 source `#[test]` fns, one per `ext` cell"),
        "",
        P.u2_source_file_wide(list(source)),
        "",
        P.u5_renames([
            ("app.<ext>", "app_await.${ext}",
             "the `for await` variant, which the source writes to the same `app.<ext>` name in a "
             "different test"),
        ]),
        "",
        NO_TEMPLATE_LITERAL,
        "",
        P.rule13_header(["kali_bin", "browser_bundle_object_string_enumeration_source",
                         "browser_bundle_object_string_enumeration_await_source",
                         helper, await_helper], runner_exemption=False),
        "This file runs no `browser_bundle_harness` step -- the build never succeeds, so the",
        "source never gets as far as writing one -- and its call chain therefore never reaches",
        "`kali_runtime_contract`'s harness helpers. Ruling 6's exemption is not stated because",
        "it has nothing to exempt here.",
        "",
        ARGV_ORDER_BUILD_ONLY,
        "",
        FAIL_CLOSED_NOTE,
        f"Both helpers' only assertion is that the process fails -- {c_fail_by_helper[helper]}",
        f"in one and {c_fail_by_helper[await_helper]} in the other, each cited from the case it",
        "actually produces. There is no stdout, stderr, exit-code or JSON claim anywhere in the",
        "file, in either output mode.",
    )

    prose = prose_of(distinct_texts(comment_blocks(text)), stem)
    cases = []
    for variant, entry, helper_name, fn_infix, describe in (
        ("plain", "app.${ext}", helper, "string_primitive_object_enumeration",
         "iterates Object.keys, Object.values and Object.entries over the string primitive 'ab' "
         "through ten root spellings each -- dotted, bracketed, single-quoted, and "
         "`Object.freeze`d nullish/logical-and/logical-or wrappers -- and checks all thirty "
         "collected arrays"),
        ("await", "app_await.${ext}", await_helper, "for_await_string_primitive_object_enumeration",
         "does the same thirty enumerations with `for await` loops inside an exported async "
         "function"),
    ):
        for json_output in (False, True):
            base = ("json_" if json_output else "") + f"build_emits_{fn_infix}_semantics"
            assert_fns(text, *[f"{base}_in_{e}_input" for e in EXTS4])
            cases.append({
                "name": base,
                "rationale": (
                    f"Migrated from browser_{stem}.rs, the four `{base}_in_*_input` fns (one per "
                    f"extension). `{helper_name}` writes the {variant} string-enumeration fixture "
                    "and runs `kali build --bundle --api browser`"
                    + (" with `--output json`" if json_output else "")
                    + f". The program {describe}. kali does not support that: the source's ONLY "
                      f"assertion is that the process fails {c_fail_by_helper[helper_name]}, so "
                      "this step carries "
                      "`exit = \"failure\"` and nothing else. Adding a diagnostic code or a "
                      "stdout claim the source never made would be a rule-2 invention, and "
                      "`exit = \"failure\"` is exactly as strong as the assertion it replaces."
                    + (" This sibling issues the `--output json` argv; the source asserts nothing "
                       "at all about the envelope, so neither does this case."
                       if json_output else "")
                    + prose),
                "steps": [build_step(entry, json_output, asserts={"exit": "failure"},
                                     json_claims=None)],
            })
    arithmetic(stem, fns=16, invocations=16, cases=len(cases), axis_len=4)
    return out(header, {"ext": EXTS4}, source, cases, non_axis=non_axis)


# ==========================================================================
# T7. browser_object_string_enumeration_harness.rs
#     16 fns / 16 invocations, [matrix] ext, fail-closed.
# ==========================================================================

@target("object_string_enumeration_harness")
def gen_object_string_enumeration_harness():
    stem = "object_string_enumeration_harness"
    text = rs(stem)
    env = assert_env_name()
    helper = "assert_browser_harness_object_string_enumeration"

    run_body = check_program("string enumeration run", fixture_in_fn(
        text, "browser_harness_object_string_enumeration_run_source"))
    # This one fixture prints NOTHING -- it is a `Kali.test(...)` block that only throws on a
    # wrong result -- so `check_program`'s default `console.log` anchor would reject a correct
    # extraction. Anchored on the construct it does have instead, rather than weakened away.
    test_body = check_program("string enumeration test", fixture_in_fn(
        text, "browser_harness_object_string_enumeration_test_source"),
        must_contain="Kali.test('browser object string enumeration', () => {")
    if run_body == test_body:
        raise AssertionError("the run and test fixtures are identical -- the split is pointless")
    source = {"main.${ext}": run_body, "smoke.test.${ext}": test_body}

    c_write = cite(text, '.expect("write source")')
    c_env = cite(text, "kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV")
    c_fail = cite(text, "assert!(!output.status.success()")

    blocks = blocks_in_fn(text, helper)
    non_axis = CLAIM_FREE_BLOCK
    header = hdr(
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        rule12_block(stem, blocks, reaching=UNIVERSAL_HELPER.format(helper=helper)),
        "",
        matrix_block(
            test_fns=16, invocations=16, cases=4, axis="ext", values=EXTS4,
            helpers=[(helper, 16,
                      "command(run/test) x ext(js/ts/jsx/tsx) x json_output(false/true), a "
                      "complete cross product. Every `#[test]` fn is one unlooped call and the "
                      "file contains no loop at all; both fixture builders are parameterless, "
                      "so `ext` really is uniform.")],
            non_axis_lines=non_axis),
        "",
        P.rule6_matrix_fold("4 source `#[test]` fns, one per `ext` cell"),
        "",
        P.u2_source_file_wide(list(source)),
        "",
        NO_TEMPLATE_LITERAL,
        "",
        P.rule13_header(["kali_bin", "browser_harness_object_string_enumeration_run_source",
                         "browser_harness_object_string_enumeration_test_source", helper],
                        runner_exemption=False),
        "This file runs no `browser_bundle_harness` step, so its call chain never reaches",
        "`kali_runtime_contract`'s two harness helpers and ruling 6's exemption has nothing",
        "to exempt here; it is not stated.",
        "",
        ARGV_ORDER_HARNESS_ONLY,
        f"The env value is `{env}`, read from the",
        "`kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV` constant the source passes by",
        f"name {c_env} rather than assumed.",
        "",
        FAIL_CLOSED_NOTE,
        f"The helper writes whichever fixture its caller passed {c_write} and its only",
        f"assertion is at {c_fail}. There is no stdout, stderr, exit-code or JSON claim",
        "anywhere in the file, in either output mode.",
    )

    prose = prose_of(distinct_texts(blocks), stem)
    cases = []
    for command, entry in (("run", "main.${ext}"), ("test", "smoke.test.${ext}")):
        for json_output in (False, True):
            base = (("json_" if json_output else "") + command
                    + "_supports_string_primitive_iteration_when_browser_harness_is_configured")
            assert_fns(text, *[f"{base}_in_{e}_input" for e in EXTS4])
            cases.append({
                "name": base,
                "rationale": (
                    f"Migrated from browser_{stem}.rs, the four `{base}_in_*_input` fns (one per "
                    f"extension). `{helper}` writes the string-enumeration {command} fixture "
                    f"{c_write} and asks `kali` to {command} it against the browser API surface "
                    "with the browser harness backed by node. The program iterates Object.keys, "
                    "Object.values and Object.entries over the string primitive 'ab' through ten "
                    "root spellings each -- dotted, bracketed, single-quoted, and "
                    "`Object.freeze`d nullish/logical-and/logical-or wrappers -- and checks all "
                    "thirty collected arrays. kali does not support that: the source's ONLY "
                    f"assertion is that the process fails {c_fail}, so this step carries "
                    "`exit = \"failure\"` and nothing else. Adding a diagnostic code or a stdout "
                    "claim the source never made would be a rule-2 invention, and "
                    "`exit = \"failure\"` is exactly as strong as the assertion it replaces."
                    + (" This sibling issues the `--output json` argv; the source asserts nothing "
                       "at all about the envelope, so neither does this case."
                       if json_output else "")
                    + prose),
                "steps": [harness_cli_step(command, entry, json_output,
                                           asserts={"exit": "failure"}, json_claims=None)],
            })
    arithmetic(stem, fns=16, invocations=16, cases=len(cases), axis_len=4)
    return out(header, {"ext": EXTS4}, source, cases, non_axis=non_axis)


# ==========================================================================
# T8. browser_object_values_harness.rs
#     36 fns / 64 invocations, [matrix] ext, fail-closed, eight programs.
# ==========================================================================

@target("object_values_harness")
def gen_object_values_harness():
    stem = "object_values_harness"
    text = rs(stem)
    env = assert_env_name()
    helper = "assert_browser_harness_object_values"
    spread_helper = "assert_browser_harness_object_values_spread"
    spread_builder = "browser_harness_object_values_spread_source"
    frozen_builder = "browser_harness_object_values_frozen_spread_source"

    plain_run = check_program("object values run", fixture_in_fn(
        text, "browser_harness_object_values_run_source"))
    plain_test = check_program("object values test", fixture_in_fn(
        text, "browser_harness_object_values_test_source"))
    global_run = check_program("global object values run", fixture_in_fn(
        text, "browser_harness_global_object_values_run_source"))
    global_test = check_program("global object values test", fixture_in_fn(
        text, "browser_harness_global_object_values_test_source"))
    spread_run = check_program("values spread run", fixture_in_fn(text, spread_builder, index=1))
    spread_test = check_program("values spread test", fixture_in_fn(text, spread_builder, index=0))
    _run, ws_run, lead_run, occ_run = assert_frozen_pair(
        "values spread run", text, C.CAP_VALUES_SPREAD_RUN_PLAIN,
        C.CAP_VALUES_SPREAD_RUN_FROZEN, plain_literal=spread_run, replace_fn=frozen_builder)
    _test, ws_test, lead_test, occ_test = assert_frozen_pair(
        "values spread test", text, C.CAP_VALUES_SPREAD_TEST_PLAIN,
        C.CAP_VALUES_SPREAD_TEST_FROZEN, plain_literal=spread_test, replace_fn=frozen_builder)
    ws_active = ws_run or ws_test
    lead, occ = P.assert_identical("needle indent", lead_run, lead_test), max(occ_run, occ_test)

    source = {
        "main.${ext}": plain_run,
        "main_global.${ext}": global_run,
        "main_spread.${ext}": C.CAP_VALUES_SPREAD_RUN_PLAIN,
        "main_frozen_spread.${ext}": C.CAP_VALUES_SPREAD_RUN_FROZEN,
        "smoke.test.${ext}": plain_test,
        "smoke_global.test.${ext}": global_test,
        "smoke_spread.test.${ext}": C.CAP_VALUES_SPREAD_TEST_PLAIN,
        "smoke_frozen_spread.test.${ext}": C.CAP_VALUES_SPREAD_TEST_FROZEN,
    }
    renamed = [k for k in source if k not in ("main.${ext}", "smoke.test.${ext}")]
    u5_check(source, renamed)

    c_fail = cite(text, "assert!(!output.status.success()", occurrence=1, expect=4)
    FROZEN_RUN_FN = ("run_supports_frozen_object_values_spread_iteration_when_browser_harness"
                     "_is_configured")
    FROZEN_TEST_FN = ("test_supports_frozen_object_values_spread_iteration_when_browser_harness"
                      "_is_configured")
    # FOUR sites spell it, in two helpers and two inlined `#[test]` fns, so each case cites the
    # one it actually reaches rather than whichever comes first in the file.
    c_fail_at = {k: cite_in(text, k, "assert!(!output.status.success()")
                 for k in (helper, spread_helper, FROZEN_RUN_FN, FROZEN_TEST_FN)}
    c_env = cite(text, "kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV", occurrence=1,
                 expect=4)
    c_replace = cite(text, f"    {spread_builder}(test_mode).replace(")

    b_values = blocks_in_fn(text, helper)
    b_spread = blocks_in_fn(text, spread_helper)
    b_frozen_run = blocks_in_fn(
        text, "run_supports_frozen_object_values_spread_iteration_when_browser_harness_is_configured")
    b_frozen_test = blocks_in_fn(
        text, "test_supports_frozen_object_values_spread_iteration_when_browser_harness_is_configured")
    all_blocks = comment_blocks(text)


    values_prose = prose_of(distinct_texts(b_values), stem)
    spread_prose = prose_of(distinct_texts(b_spread), stem)
    frozen_prose = prose_of(distinct_texts(b_frozen_run + b_frozen_test), stem)

    cases = []
    unlooped = [
        ("object_values_iteration", "main.${ext}", "smoke.test.${ext}",
         "iterates Object.values over a two-key object reached through an alias binding and "
         "prints one line", ""),
        ("direct_object_values_iteration", "main_global.${ext}", "smoke_global.test.${ext}",
         "iterates Object.values through ten `globalThis`-rooted spellings -- dotted, mixed, "
         "bracketed, single-quoted and frozen parenthesized-receiver -- and checks every "
         "collected array",
         " MIGRATION NOTE (controller ruling 8): the fn name says " + Q + "direct" + Q
         + ", but the fixture it "
         "passes is the globalThis-ROOTED program; the source is not corrected, and the "
         "discrepancy is recorded here so the case file preserves what the source actually did."),
    ]
    for infix, run_entry, test_entry, describe, note in unlooped:
        for command, entry in (("run", run_entry), ("test", test_entry)):
            for json_output in (False, True):
                base = (("json_" if json_output else "") + command + f"_supports_{infix}"
                        "_when_browser_harness_is_configured")
                assert_fns(text, *[f"{base}_in_{e}_input" for e in EXTS4])
                cases.append({
                    "name": base,
                    "rationale": (
                        f"Migrated from browser_{stem}.rs, the four `{base}_in_*_input` fns (one "
                        f"per extension). `{helper}` writes the fixture and asks `kali` to "
                        f"{command} it against the browser API surface with the browser harness "
                        "backed by node"
                        + (" with `--output json`" if json_output else "")
                        + f". The program {describe}. kali does not support that: the source's "
                          f"ONLY assertion is that the process fails {c_fail_at[helper]}, so "
                          "this step carries `exit = \"failure\"` and nothing else. Adding a "
                          "diagnostic "
                          "code or a stdout claim the source never made would be a rule-2 "
                          "invention, and `exit = \"failure\"` is exactly as strong as the "
                          "assertion it replaces." + note
                        + values_prose),
                    "steps": [harness_cli_step(command, entry, json_output,
                                               asserts={"exit": "failure"}, json_claims=None)],
                })

    looped = [
        ("run_supports_object_values_spread_iteration_when_browser_harness_is_configured",
         "run", "main_spread.${ext}", spread_helper, spread_prose, "spread",
         c_fail_at[spread_helper]),
        ("test_supports_object_values_spread_iteration_when_browser_harness_is_configured",
         "test", "smoke_spread.test.${ext}", spread_helper, spread_prose, "spread",
         c_fail_at[spread_helper]),
        (FROZEN_RUN_FN, "run", "main_frozen_spread.${ext}", None, frozen_prose, "frozen spread",
         c_fail_at[FROZEN_RUN_FN]),
        (FROZEN_TEST_FN, "test", "smoke_frozen_spread.test.${ext}", None, frozen_prose,
         "frozen spread", c_fail_at[FROZEN_TEST_FN]),
    ]
    for fn, command, entry, producer, prose, variant, fail_cite in looped:
        assert_fns(text, fn)
        for json_output in (False, True):
            cases.append({
                "name": fn + ("_json" if json_output else "_text"),
                "rationale": (
                    f"Migrated from browser_{stem}.rs, the source fn `{fn}`, its "
                    + ("`--output json`" if json_output else "text")
                    + " half (rule 5: that fn loops all four extensions and makes two "
                      "independent invocations per extension, so it becomes two named siblings "
                      "carrying the loop coordinate rather than one folded case). "
                    + (f"`{producer}` writes" if producer else
                       "The fn builds its own `Command` rather than routing through "
                       f"`{spread_helper}`, and writes")
                    + f" the {variant} fixture and asks `kali` to {command} it against the "
                      "browser API surface with the browser harness backed by node. The program "
                      "spreads Object.values over an Object.fromEntries result through nine "
                      "bracketed, single-quoted and mixed root spellings and checks every "
                      "collected array"
                    + (", with the fromEntries result itself wrapped in `Object.freeze(...)`"
                       if variant.startswith("frozen") else "")
                    + f". kali does not support that: the source's ONLY assertion is that the "
                      f"process fails {fail_cite}, so this step carries `exit = \"failure\"` "
                      "and nothing else."
                    + prose),
                "steps": [harness_cli_step(command, entry, json_output,
                                           asserts={"exit": "failure"}, json_claims=None)],
            })

    non_axis = CLAIM_FREE_BLOCK
    header = hdr(
        extra_ok_block([(v, P.EXTRA_OK_U5_RENAME) for v in expanded(renamed)]),
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        rule12_block(stem, all_blocks, reaching=(
            "The blocks do NOT all sit in one universal helper: two live in the file's two "
            "assert helpers and two more are inlined in the two frozen-spread `#[test]` fns "
            "that build their own `Command`. Each is therefore carried only into the cases its "
            "producer reaches -- see the U6 paragraph below.")),
        "",
        u6_partial_note([
            (f"`{helper}`", distinct_texts(b_values)[0]),
            (f"`{spread_helper}`", distinct_texts(b_spread)[0]),
            ("the two inlined frozen-spread `#[test]` fns", distinct_texts(b_frozen_run)[0]),
        ], cases),
        "",
        matrix_block(
            test_fns=36, invocations=64, cases=16, axis="ext", values=EXTS4,
            helpers=[
                (helper, 32,
                 "program(object-values/global-object-values) x command(run/test) x "
                 "ext(js/ts/jsx/tsx) x json_output(false/true) -- thirty-two unlooped `#[test]` "
                 "fns, one per cell"),
                (spread_helper, 16,
                 "two `#[test]` fns, each looping ext(4) and calling the helper twice per "
                 "extension (json_output false then true)"),
                ("the two inlined frozen-spread `#[test]` fns", 16,
                 "each looping ext(4) x json_output(2) around its own `Command` builder rather "
                 "than routing through the helper"),
            ],
            non_axis_lines=non_axis),
        "",
        P.rule6_matrix_fold(
            "either 4 source `#[test]` fns (one per `ext` cell) for the thirty-two unlooped fns, "
            "or one quarter of a single looping fn's invocations for the four looped ones"),
        "",
        P.u2_source_file_wide(sorted(source)),
        "",
        P.u5_renames(
            [(("main.<ext>" if k.startswith("main") else "smoke.test.<ext>"), k, why)
             for k, why in [
                 ("main_global.${ext}", "the globalThis-rooted run program"),
                 ("main_spread.${ext}", "the spread-iteration run program"),
                 ("main_frozen_spread.${ext}", "the frozen spread-iteration run program"),
                 ("smoke_global.test.${ext}", "the globalThis-rooted test program"),
                 ("smoke_spread.test.${ext}", "the spread-iteration test program"),
                 ("smoke_frozen_spread.test.${ext}",
                  "the frozen spread-iteration test program"),
             ]],
            collision="eight different program texts to just two filenames"),
        "The `.test.` infix is preserved in every renamed test entry, because `kali test` selects",
        "its entry by that name shape.",
        "",
        NO_TEMPLATE_LITERAL,
        "",
        "RULE 8 / RULE 9 -- TWO OF THESE EIGHT PROGRAMS ARE NOT STRING LITERALS.",
        f"{c_replace} builds the frozen spread variants by a str::replace off the plain ones, so",
        "there is no literal for the lexer to extract and every other route is hand-derivation --",
        "exactly the trap rule 8 exists to prevent. Both frozen texts are therefore the",
        "BYTE-EXACT OUTPUT of executing the real code (see `batch7b_captures.py` for the capture",
        "procedure).",
        wrap(f"The replace needle also carries {lead} LEADING SPACE(S) -- measured, and "
             "compared against the width the shipped captures were taken at, so a reindented "
             "source fails the generator rather than leaving this sentence wrong. Whether that "
             "indentation is load-bearing is MEASURED too, not asserted: "
             + ("it is -- the stripped needle selects a different span."
                if ws_active else
                f"it is not. The needle occurs {occ} time(s) in the plain text and the stripped "
                "form selects the same span, so the indentation hazard is LATENT here, not the "
                "operative reason. The check stays as a staleness tripwire."), 86),
        "The generator re-proves each capture against this source before emitting it: the plain",
        "capture must be byte-identical to the literal the lexer extracts from the `.rs`, the",
        "needle and replacement are read out of the `.rs` rather than restated, and applying",
        "them to the plain capture must reproduce the frozen capture exactly.",
        "",
        P.rule13_header(["kali_bin", "browser_harness_object_values_run_source",
                         "browser_harness_object_values_test_source",
                         "browser_harness_global_object_values_run_source",
                         "browser_harness_global_object_values_test_source",
                         spread_builder, frozen_builder, helper, spread_helper],
                        runner_exemption=False),
        "This file runs no `browser_bundle_harness` step, so its call chain never reaches",
        "`kali_runtime_contract`'s two harness helpers and ruling 6's exemption has nothing",
        "to exempt here; it is not stated.",
        "",
        ARGV_ORDER_HARNESS_ONLY,
        f"The env value is `{env}`, read from the",
        "`kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV` constant the source passes by",
        f"name {c_env} rather than assumed. The two inlined frozen-spread fns build the same",
        "argv by hand, in the same order.",
        "",
        FAIL_CLOSED_NOTE,
        f"Every one of the {len(cases)} cases asserts nothing but process failure, and the four",
        "sites that spell it are cited from the cases they actually produce rather than pooled:",
        f"  * {c_fail_at[helper]} -- the two unlooped programs;",
        f"  * {c_fail_at[spread_helper]} -- the spread programs;",
        f"  * {c_fail_at[FROZEN_RUN_FN]} and {c_fail_at[FROZEN_TEST_FN]} -- the two frozen-spread",
        "    fns, which inline their own `Command`.",
        "There is no stdout, stderr, exit-code or JSON claim anywhere in the file, in either",
        "output mode, so the `--output json` siblings assert nothing the text siblings do not.",
        "",
        P.migration_note_stale_fn_name(
            "run_supports_direct_object_values_iteration_when_browser_harness_is_configured"
            "_in_js_input",
            "its name says " + Q + "direct" + Q + ", but the fixture it passes is "
            "`browser_harness_global_object_values_run_source` -- the globalThis-ROOTED program, "
            "which reaches Object.values through `globalThis.Object` in ten spellings and is the "
            "opposite of a direct call. All sixteen `*_direct_object_values_iteration_*` fns in "
            "this source share the mistake."),
    )

    if len(cases) != 16:
        raise AssertionError(f"expected 16 cases, built {len(cases)}")
    arithmetic(stem, fns=36, invocations=64, cases=len(cases), axis_len=4)
    return out(header, {"ext": EXTS4}, source, cases, non_axis=non_axis)


# ==========================================================================
# T9. browser_object_values_iteration.rs
#     6 fns / 16 invocations, [matrix] ext, fail-closed + rule-11 E5506 OR.
# ==========================================================================

@target("object_values_iteration")
def gen_object_values_iteration():
    stem = "object_values_iteration"
    text = rs(stem)
    direct_helper = "assert_browser_bundle_direct_object_values_iteration"
    global_helper = "assert_browser_bundle_global_object_values_iteration"

    direct = check_program("direct object values", fixture_in_fn(
        text, "browser_bundle_direct_object_values_iteration_source"),
        must_contain="function browserDirectObjectValuesIteration()")
    global_body = check_program("global object values", fixture_in_fn(
        text, "browser_bundle_global_object_values_iteration_source"),
        must_contain="function browserGlobalObjectValuesIteration()")
    source = {"app.${ext}": global_body, "app_direct.${ext}": direct}
    u5_check(source, ["app_direct.${ext}"])

    needle = "E5506"
    c_fail = cite(text, "assert!(!output.status.success()", occurrence=1, expect=2)
    c_or = cite(text, 'stderr.contains("E5506") || stdout.contains("E5506")', occurrence=1,
                expect=2)
    # Both helpers spell both constructs, so each case cites ITS OWN helper rather than
    # whichever one happens to come first in the file.
    c_fail_by_helper = {h: cite_in(text, h, "assert!(!output.status.success()")
                        for h in (direct_helper, global_helper)}
    c_or_by_helper = {h: cite_in(text, h, 'stderr.contains("E5506") || stdout.contains("E5506")')
                      for h in (direct_helper, global_helper)}
    disjunction = ('`assert!(stderr.contains("E5506") || stdout.contains("E5506"), '
                   '"stdout: {stdout}\\nstderr: {stderr}")`')

    streams = {}
    for json_output in (False, True):
        probes = []
        for key in ("app.${ext}", "app_direct.${ext}"):
            for ext in EXTS4:
                entry = key.replace("${ext}", ext)
                argv = ["build", "--bundle", "--api", "browser"]
                if json_output:
                    argv += ["--output", "json"]
                argv += [entry]
                probes.append((entry, source[key], argv, needle, None))
        streams[json_output] = _stream(f"{stem} json={json_output}", probes)
    if streams[False] == streams[True]:
        raise AssertionError(
            "the two output modes resolve to the same stream; the per-mode split below would be "
            "describing a distinction this binary does not make")

    header = hdr(
        extra_ok_block([(v, P.EXTRA_OK_U5_RENAME) for v in expanded(["app_direct.${ext}"])]),
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        rule12_block(stem, comment_blocks(text), reaching=(
            "Both blocks are the same fail-closed re-pin note and they sit in the file's two "
            "assert helpers, `" + direct_helper + "` and `" + global_helper + "`, which between "
            "them produce every `[[case]]` below -- so U6's bottom-up attribution puts the text "
            "in every rationale. That is not the over-attribution U6 forbids; here the two "
            "helpers partition the file and carry identical prose.")),
        "",
        matrix_block(
            test_fns=6, invocations=16, cases=4, axis="ext", values=EXTS4,
            helpers=[
                (global_helper, 8,
                 "ext(js/ts/jsx/tsx) x json_output(false/true), reached through four `#[test]` "
                 "fns -- a js one and a ts/jsx/tsx loop, in each output mode"),
                (direct_helper, 8,
                 "the same eight cells, reached through two `#[test]` fns that each loop all "
                 "four extensions"),
            ],
            non_axes=("json_output",)),
        "",
        P.rule6_matrix_fold(
            "either 2 source `#[test]` fns (a js one plus a ts/jsx/tsx loop) for the "
            "globalThis-rooted program, or one looping fn for the direct one"),
        "",
        P.u2_source_file_wide(list(source)),
        "",
        P.u5_renames([
            ("app.<ext>", "app_direct.${ext}",
             "the direct-literal program, which the source writes to the same `app.<ext>` name "
             "in a different test"),
        ]),
        "",
        NO_TEMPLATE_LITERAL,
        "",
        P.rule13_header(["kali_bin", "browser_bundle_direct_object_values_iteration_source",
                         "browser_bundle_global_object_values_iteration_source",
                         direct_helper, global_helper], runner_exemption=False),
        "This file runs no `browser_bundle_harness` step -- the build never succeeds, so the",
        "source never gets as far as writing one -- and its call chain therefore never reaches",
        "`kali_runtime_contract`'s harness helpers. Ruling 6's exemption is not stated because",
        "it has nothing to exempt here.",
        "",
        ARGV_ORDER_BUILD_ONLY,
        "",
        FAIL_CLOSED_NOTE,
        f"Both helpers assert the process fails -- {c_fail_by_helper[direct_helper]} in one and",
        f"{c_fail_by_helper[global_helper]} in the other, each cited from the case it actually",
        "produces -- and both make ONE further claim, which is OR-SHAPED.",
        "",
        rule11_block(disjunction=disjunction, cite_text=c_or,
                     per_mode=[("text mode", streams[False]),
                               ("`--output json`", streams[True])],
                     needle=needle, also_true=[], needles=[needle]),
        "The json cases assert the code as a plain substring rather than as `json.errors.0.code`:",
        "the source spells it as a substring search over raw stdout, and mirroring the source is",
        "ruling 3.",
    )

    prose = prose_of(distinct_texts(comment_blocks(text)), stem)
    cases = []
    for infix, entry, helper_name, fn_names, describe in (
        ("global_object_values_iteration", "app.${ext}", global_helper,
         ("{p}build_emits_global_object_values_iteration_semantics_in_js_input",
          "{p}build_emits_global_object_values_iteration_semantics_in_ts_jsx_tsx_input"),
         "iterates Object.values through ten `globalThis`-rooted spellings -- dotted, mixed, "
         "bracketed, single-quoted and frozen parenthesized-receiver -- and checks every "
         "collected array"),
        ("direct_object_values_iteration", "app_direct.${ext}", direct_helper,
         ("{p}build_emits_direct_object_values_iteration_semantics_in_js_ts_jsx_tsx_input",),
         "iterates Object.values over object literals passed directly, through eleven root "
         "spellings including three frozen parenthesized-receiver forms"),
    ):
        for json_output in (False, True):
            prefix = "json_" if json_output else ""
            fns = [n.format(p=prefix) for n in fn_names]
            assert_fns(text, *fns)
            stream = streams[json_output]
            cases.append({
                "name": f"{prefix}build_emits_{infix}",
                "rationale": (
                    f"Migrated from browser_{stem}.rs, "
                    + ("the two fns `" + fns[0] + "` and `" + fns[1] + "` (a js one and a "
                       "ts/jsx/tsx loop, which between them cover this matrix's four cells)"
                       if len(fns) == 2 else
                       "the fn `" + fns[0] + "`, which loops all four extensions")
                    + f". `{helper_name}` writes the fixture and runs "
                      "`kali build --bundle --api browser`"
                    + (" with `--output json`" if json_output else "")
                    + f". The program {describe}. kali fails closed on it: the source asserts the "
                      f"process fails {c_fail_by_helper[helper_name]} and that the diagnostic "
                      f"code {needle} appears."
                    + rule11_rationale(disjunction, c_or_by_helper[helper_name], stream,
                                       needle, [])
                    + prose),
                "steps": [build_step(entry, json_output,
                                     asserts={"exit": "failure",
                                              f"{stream}_contains": [needle]},
                                     json_claims=None)],
            })
    arithmetic(stem, fns=6, invocations=16, cases=len(cases), axis_len=4)
    return out(header, {"ext": EXTS4}, source, cases)


# ==========================================================================
# T10. browser_object_values_spread_bundle.rs
#      8 fns / 8 invocations, [matrix] ext, build succeeds, harness fails closed.
# ==========================================================================

@target("object_values_spread_bundle")
def gen_object_values_spread_bundle():
    stem = "object_values_spread_bundle"
    text = rs(stem)
    helper = "assert_browser_bundle_object_values_spread"

    program = check_program("values spread bundle", fixture_in_fn(
        text, "browser_bundle_object_values_spread_source"),
        must_contain="function browserObjectValuesSpreadIteration()")
    source = {"app.${ext}": program}
    harness_body = check_program(
        "harness body", fixture_starting(text, helper, "const mod = await import("),
        must_contain="await mod.browserObjectValuesSpreadIteration();")

    # Two lines carry this construct -- the build's `assert!(output.status.success(),` and the
    # harness's `assert!(!output.status.success(),` -- so the occurrence is picked explicitly.
    c_build_ok = cite(text, "output.status.success(),", occurrence=1, expect=2)
    c_meta = cite(text, 'assert_eq!(metadata["apiSurface"], "browser")')
    c_payload = cite(text, 'assert_eq!(payload["bundleFormat"], "esm")')
    c_errors = cite(text, 'assert!(envelope["errors"]')
    c_fail = cite(text, "assert!(!output.status.success()")

    blocks = blocks_in_fn(text, helper)
    header = hdr(
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        rule12_block(stem, blocks, reaching=UNIVERSAL_HELPER.format(helper=helper)),
        "",
        matrix_block(
            test_fns=8, invocations=8, cases=2, axis="ext", values=EXTS4,
            helpers=[(helper, 8,
                      "ext(js/ts/jsx/tsx) x json_output(false/true), eight unlooped `#[test]` "
                      "fns; the single fixture builder is parameterless, so `ext` is uniform")],
            non_axes=("json_output",)),
        "",
        P.rule6_matrix_fold("4 source `#[test]` fns, one per `ext` cell"),
        "",
        P.u2_source_file_wide(list(source)),
        "",
        NO_TEMPLATE_LITERAL,
        "",
        P.rule13_header(["kali_bin", "browser_bundle_object_values_spread_source", helper],
                        runner_exemption=True),
        "",
        ARGV_ORDER_BUILD_ONLY,
        "",
        RUNNER_HARNESS_STEP,
        "",
        "ASSERTION SHAPE, mirrored from the source and nothing more.",
        f"  * `exit = \"success\"` on the build {c_build_ok}.",
        "  * In json mode, the envelope's schemaVersion/command/success/exitCode, the payload's",
        f"    artifactKind/bundleFormat {c_payload}, and an empty `errors` array {c_errors}.",
        f"  * `app/app.meta.json`'s apiSurface/artifactKind {c_meta}, asserted in BOTH modes,",
        "    because the source reads that file outside the `if json_output` block.",
        f"  * The browser-bundle harness FAILS CLOSED {c_fail}, and that is the source's ONLY",
        "    claim about it -- no stdout or stderr needle at all, unlike this batch's other",
        "    succeed-then-fail bundle target. `exit = \"failure\"` is exactly as strong as the",
        "    assertion it replaces, and adding a needle the source never wrote would be a",
        "    rule-2 invention.",
    )

    prose = prose_of(distinct_texts(blocks), stem)
    cases = []
    for json_output in (False, True):
        base = ("json_" if json_output else "") + "build_emits_object_values_spread_iteration"
        assert_fns(text, *[f"{base}_in_{e}_input" for e in EXTS4])
        cases.append({
            "name": base,
            "rationale": (
                f"Migrated from browser_{stem}.rs, the four `{base}_in_*_input` fns (one per "
                f"extension). `{helper}` writes the Object.values spread fixture, builds it with "
                "`kali build --bundle --api browser`"
                + (" with `--output json`" if json_output else "")
                + ", asserts the emitted app/app.meta.json metadata, then runs the bundle glue "
                  "under the browser-bundle-harness contract backed by node. The program spreads "
                  "Object.values over three differently-spelled Object.fromEntries results "
                  "through seventeen root spellings -- dotted, mixed, bracketed, single-quoted, "
                  "frozen bracket-root and frozen parenthesized-receiver -- and checks every "
                  f"collected array. The build succeeds {c_build_ok} and the metadata pins "
                  f"apiSurface/artifactKind {c_meta}, but the emitted bundle does not run: the "
                  f"harness FAILS CLOSED {c_fail}, which is the source's only claim about it."
                + (" This sibling additionally asserts the build JSON envelope -- "
                   f"schemaVersion/command/success/exitCode, payload artifactKind/bundleFormat "
                   f"{c_payload} and an empty `errors` array {c_errors} -- rather than plain "
                   "text; output shape is not a matrix axis because it changes the assertion "
                   "shape, so it is a separate case." if json_output else "")
                + prose),
            "steps": [
                build_step("app.${ext}", json_output, asserts={"exit": "success"},
                           json_claims=envelope_build(errors=True)),
                meta_step("app"),
                harness_step("app", harness_body, {"exit": "failure"}),
            ],
        })
    arithmetic(stem, fns=8, invocations=8, cases=len(cases), axis_len=4)
    return out(header, {"ext": EXTS4}, source, cases)


# ==========================================================================
# T11. browser_object_values_spread_harness.rs
#      2 fns / 16 invocations, [matrix] ext, fail-closed + rule-11 OR.
# ==========================================================================

@target("object_values_spread_harness")
def gen_object_values_spread_harness():
    stem = "object_values_spread_harness"
    text = rs(stem)
    env = assert_env_name()
    helper = "assert_browser_harness_object_values_spread"

    run_body = check_program("values spread run", fixture_in_fn(
        text, "browser_harness_object_values_spread_run_source"))
    test_body = check_program("values spread test", fixture_in_fn(
        text, "browser_harness_object_values_spread_test_source"))
    if run_body == test_body:
        raise AssertionError("the run and test fixtures are identical -- the split is pointless")
    source = {"main.${ext}": run_body, "smoke.test.${ext}": test_body}

    c_fail = cite(text, "assert!(!output.status.success()")
    c_env = cite(text, "kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV")
    c_or = cite(text, 'stderr.contains("Uncaught Error")')

    needles = ["Uncaught Error", "unreachable"]
    disjunction = ('`assert!(stderr.contains("Uncaught Error") || '
                   'stderr.contains("unreachable") || stdout.contains("Uncaught Error") || '
                   'stdout.contains("unreachable"), "stdout: {stdout}\\nstderr: {stderr}")`')
    streams, also = cli_or(stem, source, needles,
                           [("run", "main.${ext}"), ("test", "smoke.test.${ext}")])

    blocks = blocks_in_fn(text, helper)
    non_axis = fail_closed_non_axes_with_claim(text)
    header = hdr(
        f"Migrated from tests/browser_{stem}.rs.",
        "",
        rule12_block(stem, blocks, reaching=UNIVERSAL_HELPER.format(helper=helper)),
        "",
        matrix_block(
            test_fns=2, invocations=16, cases=4, axis="ext", values=EXTS4,
            helpers=[(helper, 16,
                      "command(run/test) x ext(js/ts/jsx/tsx) x json_output(false/true). Both "
                      "`#[test]` fns loop all four extensions and both output modes, so `ext` "
                      "is uniform.")],
            non_axis_lines=non_axis),
        "",
        P.RULE6_ONE_TO_ONE,
        "That is rule 5's territory rather than rule 6's here: this source has only TWO `#[test]`",
        "fns and each makes EIGHT independent invocations, so each is split into named siblings",
        "by output mode, with the extension carried by the matrix axis.",
        "",
        P.u2_source_file_wide(list(source)),
        "",
        NO_TEMPLATE_LITERAL,
        "",
        P.rule13_header(["kali_bin", "browser_harness_object_values_spread_run_source",
                         "browser_harness_object_values_spread_test_source", helper],
                        runner_exemption=False),
        "This file runs no `browser_bundle_harness` step, so its call chain never reaches",
        "`kali_runtime_contract`'s two harness helpers and ruling 6's exemption has nothing",
        "to exempt here; it is not stated.",
        "",
        ARGV_ORDER_HARNESS_ONLY,
        f"The env value is `{env}`, read from the",
        "`kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV` constant the source passes by",
        f"name {c_env} rather than assumed.",
        "",
        FAIL_CLOSED_NOTE,
        f"The source asserts the process fails {c_fail} and makes ONE further claim, which is",
        "OR-SHAPED.",
        "",
        rule11_block(disjunction=disjunction, cite_text=c_or,
                     per_mode=[("text mode", streams[False]),
                               ("`--output json`", streams[True])],
                     needle=needles[0], also_true=also, needles=needles),
    )

    prose = prose_of(distinct_texts(blocks), stem)
    cases = []
    for command, entry, fn in (
        ("run", "main.${ext}",
         "run_supports_object_values_spread_iteration_when_browser_harness_is_configured"),
        ("test", "smoke.test.${ext}",
         "test_supports_object_values_spread_iteration_when_browser_harness_is_configured"),
    ):
        assert_fns(text, fn)
        for json_output in (False, True):
            stream = streams[json_output]
            cases.append({
                "name": fn + ("_json" if json_output else "_text"),
                "rationale": (
                    f"Migrated from browser_{stem}.rs, the source fn `{fn}`, its "
                    + ("`--output json`" if json_output else "text")
                    + " half (rule 5: that fn loops all four extensions and makes two "
                      "independent invocations per extension, so it becomes two named siblings "
                      f"carrying the loop coordinate). `{helper}` writes the Object.values "
                      f"spread {command} fixture and asks `kali` to {command} it against the "
                      "browser API surface with the browser harness backed by node. The program "
                      "spreads Object.values over three differently-spelled Object.fromEntries "
                      "results through thirteen root spellings and checks every collected array. "
                      f"kali fails closed on it: the source asserts the process fails {c_fail} "
                      "and that a runtime diagnostic appears."
                    + rule11_rationale(disjunction, c_or, stream, needles[0], also)
                    + prose),
                "steps": [harness_cli_step(
                    command, entry, json_output, json_claims=None,
                    asserts={"exit": "failure", f"{stream}_contains": [needles[0]]})],
            })
    arithmetic(stem, fns=2, invocations=16, cases=len(cases), axis_len=4)
    return out(header, {"ext": EXTS4}, source, cases, non_axis=non_axis)


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
