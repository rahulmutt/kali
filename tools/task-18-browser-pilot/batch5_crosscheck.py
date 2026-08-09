#!/usr/bin/env python3
"""Cross-file consistency gate for a batch of migrated `browser/` case files.

Batch 4's review found a defect class that every per-file gate passed: four
concurrent implementers described the same recurring fact four different ways,
and one of them described a state the file no longer had. Nothing mechanical
could see it, because no gate reads `#` header prose or `rationale` wording.
This is the missing gate. It checks the two things that failure class actually
consisted of:

  1. STRUCTURE. Every file in the batch carries the same fixed set of header
     sections, in the same order. A group that invented its own section, or
     dropped one, fails here rather than being caught by eye.

  2. CITATIONS. Every `:N` written next to a backticked code snippet must point
     at a line of the paired `.rs` that actually contains that snippet. This is
     the check that four fix rounds in this project were spent doing by hand.
     A citation whose line is out of range, or whose line does not contain the
     construct it is attached to, is a hard failure.

Both are checked against the shipped `.toml`, not against the generator, so a
generator that renders the right thing and writes the wrong file is still
caught.

Deliberate non-goal: this does not check that the prose is TRUE, only that it is
consistent and that its citations resolve. U8 is explicit that rationale prose is
audited by nothing; this narrows that gap, it does not close it.

Usage: batch5_crosscheck.py [--citations-only] STEM[=PRETRIM.rs] ...
  --citations-only skips the batch-5 header-section structure arm, so the
  citation arms can gate pilot/batch-2/3/4 pairs, whose headers predate those
  section names. Batches 6-8 should run the citation arms family-wide.
Exit 0 if every file passes, 1 otherwise.

A trimmed U4 retention pair MUST be given its pre-trim blob with `=PATH`: every
`:N` in such a case file is a pre-trim line number (its own header says so), so
resolving them against the working-tree `.rs` would report failures that are
artefacts of the trim rather than stale citations -- the exact confusion ruling
9 exists to prevent.
"""

import functools
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from enumerate_invocations import strip_block_comments_and_strings  # noqa: E402

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
TESTS = os.path.join(REPO, "crates/kali_cli/tests")
CASES = os.path.join(TESTS, "cases/browser")

# Header sections every batch-5 file carries, in order. Sections marked optional
# appear only when the file's shape calls for them, but when present they must
# appear in this relative order.
SECTIONS = [
    ("Migrated from tests/browser_", True),
    ("RULE 12", True),
    ("RULE 7 / U1", True),
    ("RULE 6", True),
    ("U2 -- `[source]` is FILE-WIDE", True),
    ("RULE 13 -- transitive helper docs", True),
    ("ARGV ORDER", True),
    ("ASSERTION SHAPE", True),
]

# A backticked snippet followed by a parenthesised or bare `:N` citation.
CITE = re.compile(r"`([^`\n]{3,120})`[^`\n]{0,40}?\(?:(\d+)(?:-(\d+))?\)?")


@functools.lru_cache(maxsize=64)
def _depths(text, n_lines):
    """Bracket depth entering each line, over MASKED text. Cached: the masker
    runs over the whole file, and a header can cite it dozens of times."""
    # LINE-PRESERVING mask. `strip_block_comments_and_strings` replaces each
    # masked span with spaces of EQUAL LENGTH, so total length is preserved --
    # but the newlines inside a multi-line string or comment become spaces, and
    # the masked text then has FEWER lines than the source (269 vs 277 on
    # browser_math_round.rs). Splitting it and indexing by line number silently
    # compares the wrong lines, which is how the first version of this fix
    # produced 1-line windows for citations sitting inside a 7-line `assert!`.
    # Restoring the newlines by position is exact and keeps the verified masker.
    blanked = strip_block_comments_and_strings(text)
    assert len(blanked) == len(text), "masker changed length; index mapping unsafe"
    masked = "".join("\n" if c == "\n" else b
                     for c, b in zip(text, blanked)).split("\n")
    out, d = [], 0
    for line in masked:
        out.append(d)
        d += line.count("(") + line.count("[") - line.count(")") - line.count("]")
    # The masker blanks content but can drop a trailing line; pad defensively
    # rather than letting the gate crash on a file whose last line is masked.
    while len(out) <= n_lines:
        out.append(d)
    return out


def _statement(lines, first, last):
    """The cited line(s), widened to the whole SYNTACTIC STATEMENT they sit in.

    Replaces a fixed +-3-line window. The window was the wrong instrument for a
    drift gate twice over: it let a citation that had slipped by one or two
    lines pass (the very defect this checker exists to catch), and it was still
    arbitrary -- rustfmt routinely splits one `assert!(...)` across five lines,
    so a strict single-line rule reports correct citations as broken.

    Widening by SYNTAX instead is exact in both directions. A line is a
    continuation exactly when the bracket depth entering it is non-zero, so
    expansion stops precisely at statement boundaries and can never wander into
    a neighbouring statement -- which is what an off-by-N citation lands in.
    """
    # Mask with the project's EXISTING, verified Rust masker rather than a second
    # hand-rolled one. That is the whole lesson of this bug: the first version of
    # this expander stripped only ordinary `"..."` literals, so a `(` inside a
    # `//!` comment or an r##"..."## JS fixture put the running depth permanently
    # off zero and expansion ran to both file boundaries -- 625- and 794-line
    # windows on two adjudicated retentions, strictly weaker than the +-3 window
    # it replaced. A second attempt that hand-rolled raw-string tracking was
    # WORSE (103 files ending at non-zero depth, against 9 before). The masker in
    # `enumerate_invocations` already handles strings, raw strings and comments
    # and is used by the audit tooling; reusing it is both correct and the point.
    depth_before = _depths("\n".join(lines), len(lines))
    start, end = first, min(last, len(lines))
    # Hard clamp regardless. Even with a correct masker, a parser bug must not be
    # able to produce an unbounded window again: a gate that silently widens to
    # the whole file passes everything.
    #
    # CORRECTED, batch 6 (recorded instrument defect 3). This comment used to
    # read "MAX_EXPAND is far larger than any real statement in this corpus",
    # which is FALSE: `browser_reflect_own_keys.rs:14` is a single `format!`
    # statement spanning 184 lines, and two `browser_object_has_own_harness.rs`
    # statements span 49 and 51. What actually makes the clamp sound is the
    # DIRECTION of its error, not its size:
    #
    #   * 40 is per-DIRECTION, so the widest window a one-line citation can get
    #     is 40 back + the cited span + 40 forward -- 81 lines for a single `:N`.
    #   * Truncating the window is strictness-INCREASING. A shorter window can
    #     only make the cited token harder to find, so the clamp can turn a pass
    #     into a failure but can NEVER turn a failure into a pass. It cannot
    #     produce a false green, which is the only direction a gate must not err
    #     in (ledger `progress.md:2455-2461`: "a vacuous green is a false
    #     negative, the dangerous direction").
    #
    # So a citation into the interior of one of those 184-line statements, more
    # than 40 lines from its cited anchor, is reported rather than silently
    # passed -- and the fix for such a report is to cite a line nearer the
    # construct, not to raise the clamp.
    MAX_EXPAND = 40
    while start > 1 and depth_before[start - 1] > 0 and first - start < MAX_EXPAND:
        start -= 1
    while end < len(lines) and depth_before[end] > 0 and end - last < MAX_EXPAND:
        end += 1
    # HEADER SELF-SATISFACTION (batch 6 fix round 1, finding I3). A `//!` line is
    # blanked before the needle is searched for. Without this, a citation that has
    # drifted far enough to land inside the file's own `//!` retention header is
    # checked against that header's PROSE -- and retention prose is precisely the
    # text that names, in words, the construct the citation points at. So the
    # drifted citation self-satisfies and the gate reports green.
    #
    # Not hypothetical: `browser_math_atan2_global_this_root.rs` carried a
    # citation that had slipped to `:73-85`, which is header prose containing the
    # word `assert`, and this gate passed it. It was found by reading, not by the
    # gate that exists to find it. Over six retention headers, 1-10 of each
    # header's 50-125 lines will accept a given citation as a single-line target,
    # and EVERY citation is satisfiable by pointing at its own citing line.
    #
    # Measured cost of closing it: 0 new failures across all 69 pairs and
    # retentions. A citation must point at code; there is no legitimate citation
    # into a `//!` block, so nothing correct is lost.
    return ["" if l.lstrip().startswith("//!") else l
            for l in lines[start - 1:end]]


def _header(text):
    out = []
    for line in text.split("\n"):
        if line.startswith("#"):
            out.append(line.lstrip("#").strip())
        elif line.strip():
            break
    return out


# The case format's own key names (design spec 5.4's twelve assertion keys plus
# the structural keys). A backticked snippet led by one of these is describing
# the MIGRATED form, not a construct in the `.rs`; see `_needles`.
CASE_KEYS = {
    "exit", "stdout", "stdout_contains", "stdout_absent", "stdout_count",
    "stderr", "stderr_contains", "stderr_absent", "json", "json_null",
    "json_count", "env",
    "name", "rationale", "ignore", "kind", "args", "path", "fields",
    "entry", "body", "matrix", "source", "constants", "case",
}

# String literals inside a backticked TOML snippet, as they appear as RAW TEXT
# in a `#` comment or a rationale (no TOML escape processing has run on them).
_SNIPPET_LITERAL = re.compile(r'"((?:[^"\\]|\\.)*)"')


def _distinctive(snippet):
    """A token from a backticked snippet that should appear on the cited line.

    Prose backticks (`[matrix]`, `run`, a fn name) are not code positions, so
    only snippets that look like Rust/JS constructs are checked. Returns None
    to skip.
    """
    s = snippet.strip()
    if not any(ch in s for ch in "(.["):
        return None
    if s.startswith("[") or s.startswith("--") or " " == s:
        return None
    m = re.match(r"[A-Za-z_][A-Za-z0-9_]*", s.lstrip("&*!."))
    if not m:
        return None
    tok = m.group(0)
    return tok if len(tok) >= 4 else None


# An ellipsis used as a NAME ELISION -- `test_supports_...`, `..._in_js_input` --
# where it stands for omitted identifier characters. An ellipsis that is NOT
# adjacent to an identifier character is an omitted ARGUMENT LIST
# (`source.contains(...)`, `errors.iter().all(...)`), which names a perfectly
# good construct and must stay checked. See `_needles`.
_NAME_ELISION = re.compile(r"\w\.\.\.|\.\.\.\w")

# `receiver.method(` inside a snippet. The method name is a needle in its own
# right; see M4 in `_needles`.
_METHOD = re.compile(r"\.\s*([A-Za-z_][A-Za-z0-9_]*)\s*\(")


def _needles(snippet):
    """Every token that must appear inside the cited statement. May be empty.

    Added in batch 6, when wiring this gate into `verify_pair.sh` turned its
    false positives from noise into a hard failure on SIX already-shipped,
    CORRECT pairs. Two shapes were being mishandled, both by feeding
    `_distinctive` a snippet that is not a `.rs` construct at all:

      1. A TOML KEY ASSIGNMENT -- ``stdout_contains = ["1\\n", "0\\n"]` (:116-117)`.
         The snippet is the MIGRATED form and the `:N` points at the SOURCE
         lines it was migrated from, so the key name provably never appears in
         the `.rs` and the citation was reported broken on every such header.
         The right check is not to skip it: the string literals the key carries
         ARE the source literals the citation points at, so they are required to
         appear in the cited statement. That is strictly stronger than skipping,
         and it is what the citation is actually claiming.
      2. An ELIDED FAMILY NAME -- ``test_supports_...` fns (:311, :323, ...)`.
         An ellipsis means the author is naming a group of fns, not a construct
         at a line; the citation belongs to a nearby backtick (`main.js` here)
         and binding it to the elision is a mis-bind, not a drift. Elisions are
         skipped, exactly as ordinary prose backticks already are.

    Neither change can mask a real drift: (1) replaces an unsatisfiable token
    with the snippet's own literals, and (2) only drops snippets that name no
    single position.

    CORRECTED, fix round 1 (finding I2). (2) used to be `if "..." in s`, which
    skipped a snippet containing an ellipsis ANYWHERE -- including
    `source.contains(...)`, `errors.iter().all(...)` and
    `source.contains("Object.freeze(...)")`, where the ellipsis is an omitted
    ARGUMENT LIST and the snippet names a perfectly checkable construct. That
    silently disabled 28 citation instances, three of them genuine constructs,
    and the docstring's claim that it "only drops snippets that name no single
    position" was untrue as written. The skip is now restricted to an ellipsis
    ADJACENT TO AN IDENTIFIER CHARACTER, which is what a name elision looks like
    (`test_supports_...`) and what an omitted argument list never does.

    M4, fix round 1: a `receiver.method()` snippet yields the METHOD NAME as an
    additional needle, not just the leading identifier. `stdout.lines()` used to
    reduce to the single needle `stdout`, which is satisfied by any
    `stdout.contains(...)` line in the file -- so a `stdout.lines()` citation
    could drift onto an unrelated `stdout` assertion and pass. That is exactly
    how three stale citations in `browser_math_round_global_this_root.rs`
    survived this gate (C1). Requiring `lines` as well kills the whole class
    mechanically, which is the point: fix the mechanism, not the three numbers.

    VERIFIED BY MUTATION, and the exact claim matters. Shifting the WHOLE cited
    range by +1 and by -1 on each of the six repaired citations is caught in all
    twelve cases. What is NOT caught is WIDENING a multi-line range at one end
    (`:116-117` -> `:115-117`), and that is correct rather than a gap: this arm
    resolves at enclosing-statement granularity by design (ruling 11), so a
    range that still contains the construct is a valid pointer, not drift. The
    gate checks that a citation POINTS AT the construct; it does not, and is not
    meant to, check that the range is minimal.
    """
    s = snippet.strip()
    if _NAME_ELISION.search(s):
        return []
    lead = re.match(r"([A-Za-z_][A-Za-z0-9_]*)\s*=(?!=)", s)
    if lead and lead.group(1) in CASE_KEYS:
        lits = [l for l in _SNIPPET_LITERAL.findall(s[lead.end():]) if l]
        # Only when the assignment actually carries literals. A key whose value
        # is a bare number or bool (`exit = 2`) names nothing greppable, and
        # inventing a needle for it would reintroduce the false positive.
        return lits
    tok = _distinctive(s)
    if not tok:
        return []
    # M4: every `.method(` in the snippet joins the leading identifier as a
    # needle. Short names are dropped for the same reason `_distinctive` drops
    # them -- `.all(`/`.any(` would match far too much prose to discriminate.
    methods = [m for m in _METHOD.findall(s) if len(m) >= 4 and m != tok]
    return [tok] + sorted(set(methods))


def check(spec, citations_only=False):
    stem, _, override = spec.partition("=")
    toml_path = os.path.join(CASES, f"{stem}.toml")
    rs_path = override or os.path.join(TESTS, f"browser_{stem}.rs")
    problems = []
    # The working-tree source, ALWAYS, regardless of `=PRETRIM.rs`. A retention
    # header cites its OWN file, so it must be resolved against the shipped
    # `.rs`; only the case file's citations are pre-trim. Conflating the two
    # would have made the override silently skip the retention-header arm --
    # i.e. skip exactly the place the one real drift defect occurred.
    live_path = os.path.join(TESTS, f"browser_{stem}.rs")
    if not os.path.exists(rs_path):
        return [f"{stem}: no source at {rs_path}"]
    live_lines = open(live_path).read().split("\n") if os.path.exists(live_path) else []
    rs_header = [l for l in live_lines if l.startswith("//!")]

    if not os.path.exists(toml_path):
        # A whole-file retention has no pair, but it does have a header full of
        # citations, and those are gateable on their own.
        if not rs_header:
            return [f"{stem}: no case file at {toml_path} and no retention header"]
        problems += _header_cite_arm(stem, "\n".join(rs_header), live_lines)
        print(f"{stem}: no case file (whole-file retention); "
              f"{len(rs_header)} retention-header line(s) checked, "
              f"{len(problems)} problem(s)")
        return problems

    text = open(toml_path).read()
    rs_lines = open(rs_path).read().split("\n")
    header = _header(text)
    blob = "\n".join(header)

    # Section markers are matched at the START of a header line only. Matching
    # anywhere would collide with the prose: `MATRIX_NOT_AXES` legitimately
    # contains the words "ASSERTION SHAPE" mid-sentence, and a substring search
    # reported four files as out-of-order on that alone. A checker whose false
    # positives are indistinguishable from its true ones is not a gate.
    starts = {}
    for n, line in enumerate(header):
        for marker, _ in SECTIONS:
            if line.startswith(marker) and marker not in starts:
                starts[marker] = n

    pos = -1
    for marker, required in (() if citations_only else SECTIONS):
        idx = starts.get(marker, -1)
        if idx == -1:
            if required:
                problems.append(f"{stem}: header section missing: {marker!r}")
            continue
        if idx < pos:
            problems.append(f"{stem}: header section out of order: {marker!r}")
        pos = idx

    # Citations, over the WHOLE case file (its header + every rationale) AND
    # over the source's own `//!` retention header.
    #
    # The retention-header arm was missing until the batch-5 review pointed it
    # out, and its absence mattered: the one real citation-drift defect batch 5
    # produced was in a `//!` header (rewrapping a paragraph moved every line
    # below it), and the report claimed this gate closed that class when it
    # never read those citations at all. Ruling 11 exempts `:N` figures from the
    # no-moving-numbers rule *because* they are mechanically gated, so the gate
    # has to actually resolve them for the exemption to be earned.
    problems += _cite_arm(stem, "case file", text, rs_lines)
    if rs_header:
        problems += _header_cite_arm(stem, "\n".join(rs_header), live_lines)
    print(f"{stem}: {len(header)} header line(s) "
          f"({'+ retention header' if rs_header else 'no retention header'}), "
          f"{len(problems)} problem(s)")
    return problems


BARE_CITE = re.compile(r"`:(\d+)(?:-(\d+))?`")


def _header_cite_arm(stem, body, lines):
    """Every `:N` in a retention header must resolve, and must be RESOLVABLE.

    Retention headers cite in two forms -- ``fn_name` (`:171`)` and a bare
    `at `:290` in the bundle helper` -- and only the first carries a token the
    checker can match. The bare form was silently skipped, which is how a
    deliberately drifted citation survived this gate on its first mutation test.

    Silently skipping is the wrong answer: ruling 11 exempts `:N` figures from
    the no-moving-numbers rule *because* they are mechanically gated, so a
    citation nothing can check must not be allowed to look checked. An
    unresolvable citation is reported as a problem, and the fix is to reword the
    header so the construct is named in backticks next to its line number --
    making the artifact gateable rather than making the gate blind.
    """
    out = []
    for m in BARE_CITE.finditer(body):
        first = int(m.group(1))
        end = int(m.group(2)) if m.group(2) else first
        if end > len(lines):
            out.append(f"{stem}: retention-header citation :{first} is past end of "
                       f"the source ({len(lines)} lines)")
            continue
        # The nearest backticked, non-citation token, which must be ADJACENT.
        # The window is generous (a fn name in this corpus can be 100 chars and
        # sit on the previous header line) but the token must END within 30
        # chars of the citation. Without the adjacency bound a long window
        # happily binds a citation to an unrelated fn name two sentences back
        # and reports a false pass -- which is worse than reporting nothing.
        window = body[max(0, m.start() - 200):m.start()]
        cands = [mm for mm in re.finditer(r"`([^`\n]{2,120})`", window)
                 if not mm.group(1).startswith(":")]
        needles = []
        if cands and len(window) - cands[-1].end() <= 30:
            raw_tok = cands[-1].group(1)
            # M4 (fix round 1): the SAME needle derivation the case-file arm
            # uses, so `stdout.lines()` requires `lines` here too. This arm used
            # to call `_distinctive` directly, which reduced that snippet to
            # `stdout` -- satisfied by any `stdout.contains(...)` line in the
            # file. That is exactly how C1's three stale citations, all of them
            # in a `//!` header, survived this gate.
            needles = _needles(raw_tok)
            if not needles and re.fullmatch(r"[A-Za-z_][\w]{3,}", raw_tok):
                needles = [raw_tok]
        if not needles:
            out.append(f"{stem}: retention-header citation :{first} has no adjacent "
                       f"backticked construct to resolve against -- reword so the gate "
                       f"can check it (ruling 11 exempts :N only because it is gated)")
            continue
        stmt = "\n".join(_statement(lines, first, end))
        for tok in needles:
            if tok not in stmt:
                out.append(f"{stem}: retention-header citation :{first}"
                           f"{'-' + str(end) if end != first else ''} does not contain "
                           f"{tok!r}")
    return out


def _cite_arm(stem, origin, body, lines):
    out = []
    for m in CITE.finditer(body):
        snippet, first, last = m.group(1), int(m.group(2)), m.group(3)
        end = int(last) if last else first
        # RANGE CHECK FIRST (fix round 1, finding I2). This used to sit AFTER the
        # needle gate, so a snippet the gate declined to resolve also escaped the
        # past-end-of-file check -- a citation could point at `:9999` in a
        # 300-line file and be reported as nothing at all. Whether a citation is
        # in range is knowable without any needle, so it is checked unconditionally.
        if end > len(lines):
            out.append(f"{stem}: {origin} citation :{first} for `{snippet[:40]}` is "
                       f"past end of the source ({len(lines)} lines)")
            continue
        needles = _needles(snippet)
        if not needles:
            continue
        window = "\n".join(_statement(lines, first, end))
        for tok in needles:
            if tok not in window:
                out.append(f"{stem}: {origin} citation :{first}"
                           f"{'-' + str(end) if end != first else ''} does not contain "
                           f"{tok!r} (from `{snippet[:50]}`)")
    return out


def main(argv):
    if len(argv) < 2:
        print(__doc__)
        return 2
    citations_only = "--citations-only" in argv
    all_problems = []
    for stem in [a for a in argv[1:] if not a.startswith("--")]:
        all_problems += check(stem, citations_only)
    if all_problems:
        print("\nCROSSCHECK FAILED")
        for p in all_problems:
            print(f"  {p}")
        return 1
    print("\nCROSSCHECK OK — header structure consistent, every code citation resolves")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
