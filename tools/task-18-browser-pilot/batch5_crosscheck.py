#!/usr/bin/env python3
"""Cross-file consistency gate for a batch of migrated `browser/` case files.

Batch 4's review found a defect class that every per-file gate passed: four
concurrent implementers described the same recurring fact four different ways,
and one of them described a state the file no longer had. Nothing mechanical
could see it, because no gate reads `#` header prose or `rationale` wording.
This is the missing gate. It checks what that failure class actually consisted
of, plus the gatedness arm batch 7 added when the citation arm turned out not to
read most of the family's citations at all:

  1. STRUCTURE. Every file in the batch carries the same fixed set of header
     sections, in the same order. A group that invented its own section, or
     dropped one, fails here rather than being caught by eye.

  2. CITATIONS. Every `:N` written next to a backticked code snippet must point
     at a line of the paired `.rs` that actually contains that snippet. This is
     the check that four fix rounds in this project were spent doing by hand.
     A citation whose line is out of range, or whose line does not contain the
     construct it is attached to, is a hard failure.

     A citation may name a `#[path = "..."] mod` SUBMODULE first --
     ``snippet` (build.rs:5)` -- and is then resolved against that submodule.
     U10 targets keep every `#[test]` fn in such a sibling, so a case file
     migrated from one has no other way to point at the test it came from; and
     a bare `:N` was actively wrong there, because the unqualified pattern
     matches `(build.rs:5)` as well and would have resolved line 5 of the
     CARRIER. A qualified citation naming a file that is not a submodule of the
     target is a hard failure, not a skip. Both directions are mutation-tested
     in `--selftest`.

  3. GATEDNESS (batch 7). Every citation WRITTEN must be one a reader actually
     MATCHES. `CITE` needs a backticked construct on the same line, so a `:N`
     written as bare prose matched nothing and was never read -- it reported
     `0 problem(s)` whether it was right or wrong. Ruling 11 exempts `:N` from
     the no-moving-figures rule only because it is gated, so an unread citation
     makes that exemption unearned. See `_gated_arm` and `UNGATED_REDLIST`.

All three are checked against the shipped `.toml`, not against the generator, so a
generator that renders the right thing and writes the wrong file is still
caught.

Deliberate non-goal: this does not check that the prose is TRUE, only that it is
consistent and that its citations resolve. U8 is explicit that rationale prose is
audited by nothing; this narrows that gap, it does not close it.

Usage: batch5_crosscheck.py [--citations-only] STEM[=PRETRIM.rs] ...
       batch5_crosscheck.py --selftest
  --citations-only skips the batch-5 header-section structure arm, so the
  citation arms can gate pilot/batch-2/3/4 pairs, whose headers predate those
  section names. Batches 6-8 should run the citation arms family-wide.
  --selftest is the mutation kill for the `.all`/`.any` needle blind spot batch
  6A closed, plus the submodule arm's four properties, the batch-7 `_gated_arm`
  probes and the plain-`mod` declaration predicate; see `selftest()`. Run it
  whenever `_needles`, `SNIPPET_MAX` or a citation pattern is touched.
Exit 0 if every file passes, 1 otherwise.

A trimmed U4 retention pair MUST be given its pre-trim blob with `=PATH`: every
`:N` in such a case file is a pre-trim line number (its own header says so), so
resolving them against the working-tree `.rs` would report failures that are
artefacts of the trim rather than stale citations -- the exact confusion ruling
9 exists to prevent.
"""

import collections
import functools
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from enumerate_invocations import strip_block_comments_and_strings  # noqa: E402
from submodules import declares_submodules, submodule_paths  # noqa: E402

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

# The bound on a backticked snippet that may carry a citation. ONE constant,
# used by `CITE`, `SUBMOD_CITE` and `_header_cite_arm` alike -- batch 6B raised
# the first two to 200 and left the third at 120, which is how a single notion
# ("how long a construct can a citation name?") came to have two values.
#
# CORRECTED IN BATCH 7 (item 3.1). The comment this replaces justified the 200
# with "this corpus's `#[test]` fn names reach 161 characters". 161 is not
# reproducible under any definition and was the first violation of ruling 13
# committed into source. The derivations, each command run before this sentence
# was written:
#
#   longest cited snippet in the family today -- 137 chars, in
#   `cases/browser/non_literal_iterator_sources_explicit_api.toml`; 69 cited
#   snippets exceed 120 and 0 exceed 200. Run from the repo root:
#     $ python3 -c '
#       import sys, glob, os
#       sys.path.insert(0, "tools/task-18-browser-pilot")
#       import batch5_crosscheck as X
#       lens = []
#       for p in glob.glob(X.CASES+"/*.toml") + glob.glob(X.TESTS+"/browser_*.rs"):
#           t = open(p).read()
#           if p.endswith(".rs"):
#               t = "".join(l for l in t.splitlines(True) if l.startswith("//!"))
#           for m in list(X.CITE.finditer(t)) + list(X.SUBMOD_CITE.finditer(t)):
#               lens.append((len(m.group(1)), os.path.basename(p)))
#       print(max(lens), sum(n>120 for n,_ in lens), sum(n>200 for n,_ in lens))'
#
#   longest `#[test]` fn name anywhere in `crates/kali_cli/tests` -- 226 chars:
#     $ grep -rh -A1 '^#\[test\]' crates/kali_cli/tests --include='*.rs' \
#         | grep -oP '(?<=fn )[a-z0-9_]+' | awk '{print length}' | sort -rn | head -1
#
# So the CONCLUSION the old comment drew still holds and is now derived: 120 was
# too small (it dropped 69 of the family's cited snippets, which is 69 citations
# reported as `0 problem(s)` whether right or wrong) and 200 covers every
# citation the family actually writes (0 over).
#
# What 200 does NOT do, and the old comment implied it did: cover every citation
# that COULD be written. The longest `#[test]` fn name in this tree is 226, so a
# citation naming it would still exceed the bound. That is no longer a silent
# drop, and the reason is `_gated_arm`: a snippet over the bound means `CITE`
# does not match, and an unmatched citation is now a reported UNGATED problem
# rather than nothing at all. The bound fails LOUD, which is what makes leaving
# it at 200 defensible instead of chasing the corpus's longest identifier.
SNIPPET_MAX = 200

# A backticked snippet followed by a parenthesised or bare `:N` citation.
CITE = re.compile(
    r"`([^`\n]{3,%d})`[^`\n]{0,40}?\(?:(\d+)(?:-(\d+))?\)?" % SNIPPET_MAX)

# The same, but naming a SUBMODULE FILE first: `` `snippet` (build.rs:5) ``.
#
# U10 targets keep their `#[test]` fns in `#[path = "..."] mod` siblings, so a
# case file migrated from one cites constructs in up to five different files and
# a bare `:N` is ambiguous between them. Worse than ambiguous: `CITE` matches
# `(build.rs:5)` too, and would silently resolve line 5 of the CARRIER --
# reporting a pass or a failure about a file the author never named. So the
# qualified form is recognised explicitly, and it is matched FIRST; a `CITE` hit
# starting at the same offset as a `SUBMOD_CITE` hit is that same citation seen
# through the weaker pattern and is skipped.
SUBMOD_CITE = re.compile(
    r"`([^`\n]{3,%d})`[^`\n]{0,40}?\(?([A-Za-z0-9_]+\.rs):(\d+)(?:-(\d+))?\)?"
    % SNIPPET_MAX)


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


def _needle_found(tok, stmt):
    """Is `tok` present in `stmt` AS ITS OWN TOKEN?

    N2 (batch 7 fix round 2). Needles used to be tested with a plain `in`, and
    for the bare-identifier tier that is the exact defect the tier was added to
    catch, living inside the tier itself: a cited fn name that is a PREFIX of a
    sibling's binds to the sibling. Demonstrated on the shipped tree --
    `math_pow_alias_bundle.toml` cites `assert_browser_bundle_math_pow_alias`
    at its own declaration `:39`, and the citation could be moved to `:48` (the
    declaration of a DIFFERENT function,
    `assert_browser_bundle_math_pow_alias_with_source`) or to `:176` (a call
    site of that different function) with the gate reporting `0 problem(s)`
    both times. That turns a declared-silent citation into an
    apparently-resolved one, which is strictly worse than leaving it declared.

    Applied to EVERY identifier-shaped needle, not just the bare tier, because
    the same substring looseness is in the leading-identifier and method needles
    (`all` matching `call`, `stdout` matching `stdout_contains`). Measured over
    every citation the sweep resolves: 0 citations pass by substring and fail
    word-bounded, so the broad form costs nothing and is strictly stronger.

    A needle that is NOT identifier-shaped keeps the substring test: a literal
    needle like `--max-threads` or `0\n` is a fragment of program text, and word
    boundaries are meaningless around it.
    """
    if BARE_IDENT.fullmatch(tok):
        return re.search(r"(?<![A-Za-z0-9_])%s(?![A-Za-z0-9_])" % re.escape(tok),
                         stmt) is not None
    return tok in stmt


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

# A snippet that is one bare identifier and nothing else, and the source items a
# citation may point at. See `_needles`' last tier: a bare identifier is a needle
# only when the source DEFINES a name of its own by that spelling.
# One identifier, whole. Used both to decide whether a snippet IS a bare
# identifier and to decide whether a needle must be matched at token boundaries;
# fix round 2 introduced a byte-identical second copy of it for the latter, which
# is one edit away from the two drifting apart (minor 5, fix round 3).
BARE_IDENT = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")
_PATH_EXPR = re.compile(r"[A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)+")
_ITEM_DECL = re.compile(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)")


@functools.lru_cache(maxsize=64)
def _source_items(text):
    """Every item name the source declares. Cached on the source text, so the
    per-citation lookup is a set membership rather than a rescan."""
    return frozenset(_ITEM_DECL.findall(text))


def _needles(snippet, source_items=None):
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
        # NO LEADING IDENTIFIER (batch 7). `_distinctive` requires one of at
        # least four characters, so a snippet that is a bare method call on an
        # elided receiver -- `.arg("--max-threads")`, `.env(...)`, `.contains`
        # -- yielded NO needles at all and the citation beside it was matched
        # but never resolved. That is the same "0 problem(s) whether it is right
        # or wrong" defect `_gated_arm` measures, one layer down: the citation
        # is visible to the reader and still nothing re-resolves it.
        #
        # Such a snippet does name a position, and it names it with two things:
        # the method name and its literal argument. Neither alone discriminates
        # (`arg` occurs on every argv line in the corpus), and `_distinctive`'s
        # own >= 4 floor exists for exactly that reason -- but the CONJUNCTION
        # does, which is the same reasoning the M4 method-needle change already
        # rests on. Literals carrying a backslash are dropped: `"0\n"` is spelled
        # `0\n` in a `#` comment and `0\\n` in a `"""` rationale, so requiring it
        # would report a correct citation as broken on one surface of two.
        #
        # Measured over the 104 shipped case files plus every retention header
        # before and after: 0 change in reported problems, so nothing correct is
        # lost.
        #
        # WHAT IT BUYS, derived rather than asserted -- and CORRECTED TWICE.
        # Batch 7 wrote "~90 citations" with no command (M4). Fix round 1
        # replaced it with 419 and a hand-written command that printed `1392 223`
        # instead, because the command silently skipped the 23 sourceless stems
        # and the two U2 splits whose source is named by `Migrated from` (N1).
        # A command that does not produce the number beside it is the same
        # defect as a number nobody can reproduce.
        #
        # Both are now regenerated by a COMMITTED instrument over one stated
        # population -- the 81 resolving specs, i.e. every spec that has a case
        # file AND a source the gate resolves against:
        #
        #     $ python3 tools/task-18-browser-pilot/citation_tiers.py --fallbacks
        #     citation matches                        3551
        #     carrying needles                        3163
        #     depending on a batch-7 fallback          448
        #
        # (Both figures moved when fix round 2 recovered the `::`-path and
        # quoted-literal classes below: 3134/419 before that change, 3163/448
        # after. Re-run the command rather than trusting either.)
        methods = sorted(set(_METHOD.findall(s)))
        if methods:
            lits = sorted({l for l in _SNIPPET_LITERAL.findall(s)
                           if l and "\\" not in l})
            return methods + lits
        # A BARE BACKTICKED IDENTIFIER, resolved ONLY when the source DEFINES AN
        # ITEM OF THAT NAME (batch 7 fix round 1, I1).
        #
        # This is the last tier of the residual, and it needed a triage rather
        # than a rule, because the obvious version of it is overwhelmingly
        # WRONG. `_distinctive` declines a snippet with no `(`, `.` or `[` on the
        # grounds that it is prose rather than a code position, and the corpus
        # bears that out: admitting every bare identifier of four or more
        # characters produces 52 failures across 24 files, and 51 of the 52 are
        # FALSE -- `expected_stdout` x16, `test` x14, `stdout_contains` x7,
        # `stdout_count` x5, `kali` x5, `json_count` x2, `json_output` x1. Every
        # one is prose naming the MIGRATED form (a case-format key), the CLI
        # (`kali`, the `test` subcommand), or a source local/parameter used as a
        # label -- none of them a construct at the cited line.
        #
        # The discriminator is not a word list. It is whether the source DEFINES
        # the name: a citation is a pointer at code, so a bare identifier names a
        # position exactly when the source has an item by that name. `fn
        # assert_browser_harness_frozen_math_sin_cos_tan` is such a name;
        # `expected_stdout`, `test`, `kali` and `stdout_contains` are not items in
        # any source in this family. That single condition removes 51 of the 52
        # and keeps the one true positive, which is a real `CITE` mis-binding
        # (M8) fixed in this same round.
        #
        # Measured family-wide through the real sweep. Regenerate from the tree
        # (`--variants` re-runs every variant against the shipped corpus):
        #     $ python3 tools/task-18-browser-pilot/citation_tiers.py --variants
        # At the time of the triage, before the two M8 mis-bindings were
        # repaired, that read 52 new failures / 24 files for the unfiltered
        # variant against 1 / 1 for this one. On the repaired corpus the same
        # command reads 50 / 23 against 0 / 0 -- the two that disappeared ARE
        # the M8 pair, which is the check that the triage found real defects
        # rather than noise.
        # and 65 citations that previously yielded NO needles at all now resolve.
        # Regenerate from the tree -- the tier is switched off and the whole
        # sweep re-run, so the figure is the tier's own contribution rather than
        # a hand-partition of the snippets:
        #     $ python3 tools/task-18-browser-pilot/citation_tiers.py --gains
        #     shipped                                  388
        #     without the source-defined-item tier     453   (gives needles to 65)
        # All 65 resolve rather than fail, across 16 distinct snippets.
        # (Fix round 1 wrote 24 here. That was the FILE count copied off the
        # "52 failures across 24 files" line two paragraphs up -- a figure
        # transplanted between two sentences about different units, which is
        # ruling 13's shape exactly.)
        # `source_items` is None only for a caller with no source in hand (the
        # gatedness-only path), where this tier is skipped rather than guessed.
        if source_items and BARE_IDENT.fullmatch(s) and len(s) >= 4:
            if s not in CASE_KEYS and s in source_items:
                return [s]
        # DECLARED BARE NEEDLES (fix round 3). See `BARE_NEEDLE_ADMITTED`.
        if s in BARE_NEEDLE_ADMITTED:
            return [s]
        # TWO MORE SHAPES THAT ARE UNAMBIGUOUSLY CODE (fix round 2). The review
        # measured the declared NO-NEEDLE tier and found the sentence describing
        # it was false: it is not all "prose that names no code position". Of the
        # 417, 276 occur verbatim in their own non-comment source. Two classes
        # inside that are mechanically recoverable, and both cost 0 new failures
        # measured across the whole sweep:
        #
        #   * a `::` PATH EXPRESSION -- `kali_runtime_contract::BROWSER_HARNESS_
        #     COMMAND_ENV`, 20 citations, which batch 7's own report happened to
        #     quote as an example of a GENUINE citation while the gate was
        #     declining to read it. `_distinctive` misses it because `::` is
        #     none of `(`, `.`, `[`. The last segment is the needle: it is the
        #     distinctive half and it is what appears at the cited line.
        #   * a snippet carrying a QUOTED LITERAL -- `if command == "test"`,
        #     `if command != "build"`, 9 citations. The literal plus the
        #     identifiers around it are exactly what the line contains.
        #     Backslash-bearing literals are excluded for the reason the
        #     no-leading-identifier tier gives: `"0\n"` is spelled differently in
        #     a `#` comment and in a multi-line TOML string.
        #
        # Together these move 29 citations out of the declared tier and into the
        # checked one (417 -> 388). What remains is genuinely unresolvable by a
        # construct search: bare `stderr`/`errors` locals, argv strings, file
        # paths, CLI subcommands and case-format keys.
        if _PATH_EXPR.fullmatch(s):
            return [s.split("::")[-1]]
        lits = sorted({l for l in _SNIPPET_LITERAL.findall(s)
                       if l and "\\" not in l and re.search(r"[A-Za-z]", l)})
        if lits:
            idents = {i for i in BARE_IDENT.findall(s) if len(i) >= 4}
            return sorted(set(lits) | idents)
        return []
    # M4: every `.method(` in the snippet joins the leading identifier as a
    # needle.
    #
    # THE LENGTH FLOOR IS GONE (batch 6A). It used to read `len(m) >= 4`, on the
    # reasoning that `.all(`/`.any(` "would match far too much prose to
    # discriminate" -- true of a needle used ALONE, which is the case
    # `_distinctive` guards, and false here: a method needle is only ever added
    # ALONGSIDE a leading identifier that `_distinctive` has already required to
    # be >= 4 chars, so the conjunction is what discriminates and the floor only
    # deleted the one token that told two sibling statements apart. The cost was
    # exactly the blind spot M4 was written to close, one method pair over:
    # `errors.iter().all(...)` and `errors.iter().any(...)` both reduced to
    # ['errors', 'iter'], so a citation could sit on either and pass.
    #
    # Demonstrated on the shipped tree and now gated by `--selftest` below:
    # moving `browser_math_pow_optional_chain_harness.rs`'s `.all` citation onto
    # the `.any` statement was NOT caught before this change and is caught after.
    # That construct is the design spec 5.11 shape carried by both of batch 6A's
    # retentions, so the blind spot sat directly under the batch that exercises
    # it most.
    #
    # A short bare needle cannot produce a FALSE RED here: the snippet is quoted
    # from the statement it cites, so every method name in it occurs in that
    # statement by construction. Measured across the full family sweep (70
    # pairs + retentions), removing the floor changes nothing -- the residual
    # stays at the 7 known-ungateable bare `:N` in
    # `browser_generator_default_export_rejection.rs`, batch 8's to reword.
    methods = [m for m in _METHOD.findall(s) if m != tok]
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
        # NO SOURCE: the `.rs` was deleted after its migration shipped. The two
        # RESOLVING arms genuinely cannot run -- there is nothing to resolve
        # against -- but the GATEDNESS arm can, because whether a citation is one
        # any reader matches is a property of the case file's own text.
        #
        # BATCH 7 FIX ROUND 1, I2. This used to return one problem and stop, and
        # `citation_sweep.sh` avoided that by SKIPPING such a stem entirely --
        # 23 of the 104 case files, `_gated_arm` never run on any of them. Two
        # ungated citations were hiding there (`bundle_cjs_source_classes.toml`
        # and `..._inherited.toml`, both `classes.rs:23-33`): unearned AND
        # undisclosed, which is the exact state item 1 exists to end. Running the
        # arm that CAN run is strictly better than skipping the file.
        if not os.path.exists(toml_path):
            return [f"{stem}: no source at {rs_path} and no case file at {toml_path}"]
        text = open(toml_path).read()
        problems = _gated_arm(stem, "case file", text)
        # THE CARVE-OUT IS DECLARED, NOT SILENT (fix round 2, N3). Returning here
        # skips `_cite_arm`, so `_NO_NEEDLE[stem]` was never incremented and
        # `NO_NEEDLE_DECLARED.get(stem, 0)` returned 0 -- equal, pass. The tier's
        # "all of it declared" was therefore true only because this branch was
        # invisible to it. Demonstrated: a resolvable-looking citation injected
        # into `object_keys_harness.toml` gave `0 problem(s)` and `CROSSCHECK OK`,
        # while the identical line in a sourced stem was reported.
        #
        # With no source there is nothing to resolve against, so EVERY citation
        # match here is unresolvable and every one is counted. That is 0 today --
        # these 23 stems carry no citation match at all, only the two red-listed
        # ungated ones -- so the equality check now passes for a MEASURED reason
        # instead of because the counter was never touched, and a citation added
        # to one of them moves the count off its declared 0 and fails.
        starts = {m.start() for m in CITE.finditer(text)}
        starts |= {m.start() for m in SUBMOD_CITE.finditer(text)}
        _NO_NEEDLE[stem] += len(starts)
        print(f"{stem}: source deleted post-migration; gatedness arm only, "
              f"{len(starts)} unresolvable citation match(es), "
              f"{len(problems)} problem(s)")
        return problems
    live_lines = open(live_path).read().split("\n") if os.path.exists(live_path) else []
    rs_header = [l for l in live_lines if l.startswith("//!")]

    if not os.path.exists(toml_path):
        # A whole-file retention has no pair, but it does have a header full of
        # citations, and those are gateable on their own.
        if not rs_header:
            return [f"{stem}: no case file at {toml_path} and no retention header"]
        problems += _header_cite_arm(stem, "\n".join(rs_header), live_lines)
        problems += _gated_arm(stem, "retention header", "\n".join(rs_header))
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
    # SUBMODULE RESOLUTION SURVIVES A `=PATH` OVERRIDE (fix round 1, I5).
    # The override's text is a git blob in a temp dir, so resolving its
    # `#[path]` declarations relative to ITSELF finds nothing, and every
    # qualified citation in the pair is then reported as naming a
    # non-submodule -- a green pair turned red by an artefact of the override,
    # which is the exact confusion `--pretrim` exists to prevent. The blob is a
    # copy of a tree file, so a TREE directory is the right base; the text still
    # comes from the override. Bases are tried in order of confidence, and if
    # the source declares `#[path]` mods and NONE resolve, that is ONE loud
    # problem rather than N misleading ones. No shipped pair needs this today;
    # batches 7-8 meet `#[path]` carriers WITH retentions, which is exactly the
    # combination.
    submodules = {}
    declares_mods = False
    if os.path.exists(rs_path):
        # ITEM 3.3 (batch 7). This used to read
        # `"#[path" in open(rs_path).read()`: a `#[path]`-ONLY substring test,
        # while `submodule_paths` also resolves plain `mod x;` / `pub mod x;`
        # chains. A plain-`mod` carrier whose submodules failed to resolve was
        # therefore judged not to declare any, fell past the single loud problem
        # batch 6B added for that case, and produced N misleading "names a file
        # that is not a submodule" problems instead.
        # `browser_cdp_smoke.rs` carries the plain-`mod` shape, so this was not
        # hypothetical. It also leaked the file handle; the text is read once
        # here and reused.
        with open(rs_path) as fh:
            rs_text = fh.read()
        declares_mods = declares_submodules(rs_text)
        bases = []
        if os.path.dirname(os.path.abspath(rs_path)) == os.path.abspath(TESTS):
            bases.append(rs_path)          # a --rs split: already a tree file
        if os.path.exists(live_path):
            bases.append(live_path)        # a --pretrim blob of a live stem
        named = _migrated_from(text)       # the case file names its own source
        if named:
            bases.append(os.path.join(TESTS, named))
        for base in bases:
            if not os.path.exists(base):
                continue
            found = submodule_paths(rs_path, base=base)
            if found:
                submodules = {os.path.basename(f): f.read_text().split("\n")
                              for f in found}
                break
    if declares_mods and not submodules:
        problems.append(
            f"{stem}: the source declares `#[path]` submodule(s) but none could be "
            f"resolved from {rs_path} -- qualified `<file>.rs:N` citations cannot "
            "be checked, so they must not be reported as unresolvable names")
    problems += _cite_arm(stem, "case file", text, rs_lines, submodules)
    problems += _gated_arm(stem, "case file", text)
    if rs_header:
        problems += _header_cite_arm(stem, "\n".join(rs_header), live_lines)
        problems += _gated_arm(stem, "retention header", "\n".join(rs_header))
    print(f"{stem}: {len(header)} header line(s) "
          f"({'+ retention header' if rs_header else 'no retention header'}), "
          f"{len(problems)} problem(s)")
    return problems


BARE_CITE = re.compile(r"`:(\d+)(?:-(\d+))?`")

# THE "WRITTEN" SIDE OF RULING 11'S WRITTEN-VS-MATCHED TEST.
#
# Hoisted out of `gen_batch6b.py`'s `assert_every_citation_is_gated` in batch 7.
# There it guarded one generator's two files; ruling 11 exempts `:N` from the
# no-moving-figures rule *family-wide*, and the exemption is conditional on the
# citation being mechanically gated -- "a pointer nothing re-resolves is a
# figure in disguise". A check that runs on two of 104 files does not earn that.
#
# This pattern is DELIBERATELY WIDER than `CITE`/`SUBMOD_CITE`/`BARE_CITE`. Its
# job is to see citations those cannot, because that is the whole defect: `CITE`
# requires a backticked construct within 40 chars on the SAME line, so a citation
# written as bare prose -- `schemaVersion (:68)`, `command (:69)` -- matches
# nothing and is never read. It does not fail; it is INVISIBLE, and reports
# `0 problem(s)` whether it is right or wrong.
#
# Shape, and why each guard is there (each one was measured over the 104 shipped
# case files plus every retention header; see the batch-7 report's derivation):
#   * the `:N` must follow whitespace, `(`, `,` or `[` -- a citation is always
#     introduced, never glued to an identifier. Without this, `1:1` in "the
#     assertion mapping stays 1:1 per trial" reads as a citation to line 1, and
#     `{"schemaVersion":1}` in a `[source]` JS fixture reads as one too.
#   * `(?![\d:])` after the number, so a version or a ratio is not truncated
#     into a citation.
#   * a leading `<file>.rs` is allowed, for the qualified `SUBMOD_CITE` form.
# Measured today: every hit of this pattern that `CITE`/`SUBMOD_CITE` does not
# cover lies inside a `#` header line or a `rationale` -- 0 in `[source]` bodies
# or anywhere else. Derivation command in the batch-7 report; re-run it before
# widening this pattern.
WRITTEN_CITE = re.compile(
    r"(?:(?<=\s)|(?<=\()|(?<=,)|(?<=\[))(?:[A-Za-z0-9_]+\.rs)?:\d+(?:-\d+)?(?![\d:])")


# RULING-9-STYLE RED-LIST for `_gated_arm`, and nothing else.
#
# Ruling 9 red-lists a gate that is EXPECTED red for a structural reason, names
# every entry, and states the reason -- rather than weakening the gate until the
# red goes away. This is the same instrument for the same shape of problem, and
# the alternative was explicitly forbidden: widening a reader pattern until
# these citations are swallowed would convert a measured gap into an invisible
# one, which is what `_gated_arm` exists to end.
#
# WHAT IS ON IT, and why each entry cannot be reworded. The gate resolves a
# citation by finding a backticked RUST CONSTRUCT in the cited statement. Every
# entry below cites a line that contains no Rust construct at all, so there is
# nothing to put in the backticks:
#
#   FIXTURE-TEXT -- the rule-12 comment inventory ("The N other `//`
#     occurrence(s) in the file") cites a `//` sitting inside a `r##"..."##` JS
#     fixture body. It is JavaScript, not Rust; `enumerate_invocations`'s masker
#     blanks it, and the statement window is the one fixture line.
#   RUST-COMMENT -- a "Rust comment block (:N)" citation, which by construction
#     points at a `//` line. `_statement` deliberately cannot resolve against
#     comment text (that is finding I3's header-self-satisfaction fix), so a
#     citation onto a comment is unresolvable by design, not by accident.
#   CONTROL-FLOW -- a citation onto a bare `if json_output {` / `for ... {`
#     line, cited precisely because of where it sits in the control flow.
#   SOURCE-DELETED -- added in batch 7's fix round 1 (I2). The cited `.rs` was
#     deleted once its migration shipped, so NO reader in the tree can resolve
#     the line number: it is recoverable only from git history. Rewording cannot
#     help -- there is nothing to point at -- and the honest disposition is to
#     declare it rather than to leave it invisible. These two were invisible for
#     a second reason as well: `citation_sweep.sh` skipped every sourceless stem
#     outright, so `_gated_arm` never ran on 23 of the 104 case files. It runs on
#     them now, and these are the only two ungated citations it found there.
#
# These 28 are what remained after 845 of the family's 873 ungated citations
# were reworded mechanically by `reword_ungated_citations.py` (873 is the
# family-wide figure over all 104 case files; the sweep's own run reported 871
# because it was skipping the two SOURCE-DELETED sites below). An entry that
# stops firing is reported as STALE by `main()` -- a red-list nobody re-checks
# is the same figure-in-disguise one level up.
UNGATED_REDLIST = {
    ("math_hypot_global_this_root", "case file", ":11"): "FIXTURE-TEXT",
    ("math_log2_log10_mixed_root", "case file", ":11"): "FIXTURE-TEXT",
    ("math_log2_log10_mixed_root", "case file", ":225"): "CONTROL-FLOW",
    ("math_pow_bracketed_root", "case file", ":245"): "FIXTURE-TEXT",
    ("math_pow_zero_exponent_non_integer_base", "case file", ":11"): "FIXTURE-TEXT",
    ("math_pow_zero_exponent_non_integer_base", "case file", ":331"): "FIXTURE-TEXT",
    ("math_round_bracketed_root", "case file", ":11"): "FIXTURE-TEXT",
    ("math_sin_cos_tan_bracketed_root", "case file", ":11"): "FIXTURE-TEXT",
    ("math_sin_cos_tan_frozen_root", "case file", ":11"): "FIXTURE-TEXT",
    ("math_sin_cos_tan_fully_bracketed_root", "case file", ":11"): "FIXTURE-TEXT",
    ("math_sin_cos_tan_zero_identities", "case file", ":11"): "FIXTURE-TEXT",
    ("math_sinh_cosh_tanh_bracketed_root", "case file", ":11"): "FIXTURE-TEXT",
    ("math_sinh_cosh_tanh_global_this_root", "case file", ":11"): "FIXTURE-TEXT",
    ("math_sqrt_cbrt_bracketed_root", "case file", ":11"): "FIXTURE-TEXT",
    ("math_sqrt_cbrt_bundle", "case file", ":11"): "FIXTURE-TEXT",
    ("math_sqrt_cbrt_frozen_aliases", "case file", ":11"): "FIXTURE-TEXT",
    ("math_sqrt_cbrt_global_this_root", "case file", ":11"): "FIXTURE-TEXT",
    ("math_unsupported_member_calls_harness_jsx_tsx", "case file", ":83"): "CONTROL-FLOW",
    ("object_computed_numeric_keys_bundle", "case file", ":11"): "FIXTURE-TEXT",
    ("object_computed_numeric_keys_bundle", "case file", ":22"): "FIXTURE-TEXT",
    ("object_entries_harness", "case file", ":160"): "RUST-COMMENT",
    ("object_entries_iteration", "case file", ":175"): "RUST-COMMENT",
    ("object_entries_iteration", "case file", ":258"): "RUST-COMMENT",
    ("object_entries_iteration", "case file", ":340"): "RUST-COMMENT",
    ("object_enumeration_finalization_bundle", "case file", ":319"): "RUST-COMMENT",
    ("object_enumeration_finalization_harness", "case file", ":414"): "RUST-COMMENT",
    ("bundle_cjs_source_classes", "case file", "classes.rs:23-33"): "SOURCE-DELETED",
    ("bundle_cjs_source_classes_inherited", "case file", "classes.rs:23-33"): "SOURCE-DELETED",
}

_REDLIST_HIT = set()

# BARE BACKTICKED SNIPPETS THIS CORPUS USES AS CONSTRUCTS, declared (fix round 3).
#
# The source-defined-item test above covers a bare identifier the source declares
# as an item. It does not cover an identifier the source only BINDS or INDEXES --
# `stderr`, `errors`, `exitCode` -- and those are the declared tier's two largest
# classes plus its fifth.
#
# WHY THIS IS A DECLARATION AND NOT A RULE, which matters because the obvious
# rule is wrong by a factor of two. Admitting every bare identifier that occurs
# in the source admits `expected_stdout`, `run`, `test` and `app/app.meta.json`
# too, and those produce 211 FALSE reds -- because in this corpus a bare backtick
# is used in two different ways and only one of them is a construct:
#
#   CONSTRUCT   ``errors` (:228)`  -> `:228` is `let errors = json["errors"]...`
#   LABEL       ``expected_stdout` = "1\n1" (:204)` -> `:204` is `"1\n1",`; the
#               citation points at the VALUE the label describes, not at the
#               identifier. Same shape: ``for `run` (:228-229)` pointing at
#               `assert_eq!(json["exitCode"], 0)`, and ``app/app.meta.json`
#               metadata (:99-100)` pointing at the metadata assertions.
#
# No lexical predicate separates a label from a construct -- the difference is
# what the author meant the backtick to do. So this is an adjudicated list, the
# same instrument as `UNGATED_REDLIST`, and each entry's cost was measured by
# admitting it alone across the whole sweep:
#
#     stderr    86 citations   0 new failures
#     errors    74 citations   0 new failures
#     exitCode  14 citations   0 new failures
#     ---- rejected, measured the same way ----
#     app/app.meta.json  31    31 new failures (a path, always a label)
#     run       22            17 new failures
#     test      19            14 new failures
#     expected_stdout 16      16 new failures
#
# Regenerate the whole partition, including which snippets are admissible at zero
# cost and which are not:
#     $ python3 tools/task-18-browser-pilot/citation_tiers.py --admissible
#
# Two further snippets measure at zero cost and are deliberately NOT here: a
# bare `"8"` (2 citations, not an identifier) and `build` (1 citation). `build`
# is the same subcommand-label shape as `run` and `test`, which fail on 31 of
# their 41 citations between them; its single instance passing is luck, not a
# property, and admitting it would encode the luck.
#
# Admitting these is a STRENGTHENING: 174 citations move from "nothing searches
# for them" to "the identifier must be at the cited line", and all 174 pass
# today, so a future drift in any of them is now caught.
BARE_NEEDLE_ADMITTED = frozenset({"stderr", "errors", "exitCode"})

# THE THIRD TIER, DECLARED (batch 7 fix round 1, I1).
#
# A citation can fail to be re-resolved in three ways, and until this round only
# two of them were governed:
#
#   1. UNGATED   -- no reader matches it at all. `_gated_arm`; red-listed above.
#   2. UNRESOLVED-- a reader matches it and the needle is absent. A hard failure.
#   3. NO-NEEDLE -- a reader matches it and `_needles` declines to derive one,
#                   so nothing is searched for. It reports clean whether it is
#                   right or wrong -- exactly the defect `_gated_arm` exists to
#                   end, reached through a different door.
#
# Tier 3 was SILENT. It is now counted per stem and declared here, because a gap
# that is named is governed and a gap that is silent is not. The declaration is
# checked for EQUALITY, not as a ceiling: a stem whose count rises has added an
# unreadable citation, and a stem whose count falls has had one become
# resolvable and must shrink its entry. Either way the number in this file and
# the number the corpus produces have to agree, which is what stops the tier
# drifting back into silence.
#
# WHAT IS ACTUALLY IN IT, measured -- and this description has now been wrong
# TWICE, both times as a false quantifier of ruling 13's own shape, so it is
# regenerated rather than characterised:
#
#     $ python3 tools/task-18-browser-pilot/citation_tiers.py --describe
#     citations in the tier                    214
#     occurring verbatim in their own source    73
#     resolving at their own cited line         14
#     pinned (both +-1 shifts lose them)         2
#     distinct snippets                         39
#     of which case-format keys (CASE_KEYS)     14
#
# Round 1 said the tier was "prose that names no code position at all". FALSE:
# 276 of the 417 then in it occurred verbatim in their own non-comment source.
# Round 2 replaced that with "none is a construct a search can pin to one
# statement". Also FALSE: 188 of the 388 resolved at their cited line and 145
# were killed by both +-1 shifts, including all 86 `stderr` and 56 of 74
# `errors`. `none` is one of ruling 13's trigger words and it was wrong.
#
# Round 3 stopped redescribing and MOVED them: `stderr`, `errors` and `exitCode`
# are declared needles now (see `BARE_NEEDLE_ADMITTED`), 174 citations at 0
# measured cost. That is why the figures above are so much smaller than the ones
# they replace -- the resolvable part of the tier left it.
#
# What is left is 214 citations across 39 snippets, and the honest statement is
# the measurement, not an adjective: 73 of them appear verbatim somewhere in
# their own source, 14 appear at their own cited line, and 2 are pinned there.
# Admitting any of them as its own needle was measured one snippet at a time and
# 37 of the 39 produce new failures -- `app/app.meta.json` 31, `run` 17, `test`
# 14, `expected_stdout` 16 -- because in every one of those the backtick is a
# LABEL and the citation points at what the label describes, not at the label.
# The two that would cost nothing (`"8"`, `build`, 3 citations between them) are
# left out for the reason `BARE_NEEDLE_ADMITTED` gives.
#
# Regenerate this dict, from the tree:
#     $ python3 tools/task-18-browser-pilot/citation_tiers.py --declare
NO_NEEDLE_DECLARED = {
    "math_asinh_acosh_atanh_identities": 1,
    "math_exp_log_mixed_root": 1,
    "math_expm1_log1p_frozen_aliases": 8,
    "math_expm1_log1p_fully_bracketed_root": 1,
    "math_expm1_log1p_global_this_root": 7,
    "math_floor_trunc_ceil_bracketed_root": 4,
    "math_fully_bracketed_root_core_suite": 1,
    "math_global_this_root_core_suite": 1,
    "math_hypot_empty_identity": 4,
    "math_hypot_frozen_aliases": 3,
    "math_imul_clz32_aliases": 2,
    "math_imul_omitted_operands": 3,
    "math_log2_log10": 4,
    "math_log2_log10_bracketed_root": 4,
    "math_log2_log10_fully_bracketed_root": 1,
    "math_log2_log10_mixed_root": 8,
    "math_max_min_frozen_aliases": 6,
    "math_pow_alias_bundle": 1,
    "math_pow_bracketed_frozen_wrapper": 1,
    "math_pow_bracketed_frozen_wrapper_bundle": 1,
    "math_pow_bracketed_root": 10,
    "math_pow_harness": 1,
    "math_pow_zero_exponent_non_integer_base": 31,
    "math_round_bracketed_root": 25,
    "math_sin_cos_tan_bracketed_root": 6,
    "math_sin_cos_tan_frozen_root": 13,
    "math_sin_cos_tan_fully_bracketed_root": 6,
    "math_sin_cos_tan_zero_identities": 2,
    "math_sinh_cosh_tanh_bracketed_root": 6,
    "math_sinh_cosh_tanh_global_this_root": 10,
    "math_sqrt_cbrt_bracketed_root": 11,
    "math_sqrt_cbrt_bundle": 1,
    "math_sqrt_cbrt_frozen_aliases": 1,
    "math_sqrt_cbrt_global_this_root": 4,
    "math_sqrt_cbrt_harness": 2,
    "math_tan_zero_identities": 3,
    "math_unsupported_member_calls_harness_jsx_tsx": 4,
    "nullish_coalescing_harness": 1,
    "number_predicates_bundle": 1,
    "number_predicates_harness": 2,
    "object_computed_numeric_keys_bundle": 1,
    "object_computed_numeric_keys_harness": 5,
    "object_entries_harness": 2,
    "object_entries_iteration": 1,
    "object_enumeration_finalization_bundle": 1,
    "object_enumeration_finalization_harness": 2,
}

_NO_NEEDLE = collections.Counter()


def _gated_arm(stem, origin, body):
    """Every citation WRITTEN must be one a citation arm actually READS.

    This is the count-vs-resolve distinction ruling 11 turns on. `_cite_arm` and
    `_header_cite_arm` answer "does this citation point at its construct?" -- but
    only for citations they can see. This arm answers the prior question: is the
    citation visible to them at all? A `:N` no reader matches is not a passing
    citation; it is an unread one, and reporting it as `0 problem(s)` is the
    false green ledger `progress.md:2455-2461` names as the dangerous direction.

    POSITIONAL, NOT A COUNT. `gen_batch6b.py` compared `len(written)` against
    `len(matched)`, which is sound on a file whose prose the generator controls
    and unsound family-wide in BOTH directions: `CITE` matches things nobody
    wrote as a citation (`ext` + `stays 1:1` reads as ``ext` ... :1`), so the two
    counts can agree while a real citation is unread. Requiring each written
    citation to be COVERED BY a match names the offending site instead, and
    cannot be satisfied by an unrelated spurious match elsewhere in the file.

    WHICH SURFACE THIS ARM ACTUALLY GUARDS (corrected in fix round 2, M6). It
    used to take an `extra_readers=(BARE_CITE,)` argument on the retention-header
    call, and both this docstring and the batch-7 report said it "runs on both
    surfaces". That reads as coverage which is not exercised: `WRITTEN_CITE`
    admits a `:N` only after whitespace, `(`, `,` or `[`, while `BARE_CITE`
    requires a backtick immediately before the `:`, so NO STRING can produce a
    `WRITTEN_CITE` hit that `BARE_CITE` covers -- the parameter was unreachable
    by construction. Measured: `WRITTEN_CITE` finds 0 hits across all 17
    retention headers, which carry 61 `BARE_CITE` citations between them.

    ONLY THE ARGUMENT WAS WRONG, NOT THE CALL (N4, fix round 3). Round 2 deleted
    the whole retention-header invocation along with the dead parameter, and that
    lost real coverage: `_header_cite_arm` iterates `BARE_CITE` only, so it sees
    a `` `:N` `` and CANNOT see an un-backticked one. Measured on
    `browser_cdp_smoke.rs` -- inserting `//! JSON envelope: schemaVersion (:8),
    command (:9)` into its header reported 2 problems before round 2 and 0 after,
    and `//! the blocking helper at (:8) is unreadable` went 1 -> 0, while the
    backticked control stayed reported both times. Those citations were then
    neither resolved, nor declared, nor reported -- and the success banner
    claiming "every one it cannot [resolve] is declared" was untrue for them.

    So the call is back at both sites, without the parameter, and it costs
    nothing: the sweep still exits 0.

    The division of labour, stated properly: on a retention header
    `_header_cite_arm` resolves the BACKTICKED citations and reports any whose
    construct is missing or absent; this arm catches the ones it cannot see at
    all, which on that surface means every un-backticked `:N`. On a case file
    there is no `BARE_CITE` reader, so this arm is the only gatedness check.

    The fix for a report here is to REWORD -- put the construct in backticks
    beside the number -- so the citation becomes resolvable. It is never to
    widen a reader pattern until the citation is swallowed: that converts a
    measured gap into an invisible one, which is the state this arm exists to
    end.
    """
    covered = [(m.start(), m.end())
               for m in list(SUBMOD_CITE.finditer(body)) + list(CITE.finditer(body))]
    out = []
    for m in WRITTEN_CITE.finditer(body):
        if any(a <= m.start() < b for a, b in covered):
            continue
        key = (stem, origin, m.group(0))
        if key in UNGATED_REDLIST:
            _REDLIST_HIT.add(key)
            continue
        line_no = body.count("\n", 0, m.start()) + 1
        start = body.rfind("\n", 0, m.start()) + 1
        end = body.find("\n", m.start())
        context = body[start:end if end != -1 else len(body)].strip()
        out.append(
            f"{stem}: {origin} citation `{m.group(0)}` is UNGATED -- no citation "
            f"reader matches it, so nothing re-resolves it and it reports clean "
            f"whether it is right or wrong (ruling 11). At {origin} line "
            f"{line_no}: {context[:110]!r}. Reword so a backticked construct sits "
            f"beside the number.")
    return out

# The `Migrated from tests/browser_X.rs` line every case file's header opens
# with. It is the only in-tree statement of which source a case file came from,
# and for a U2 SPLIT (whose stem deliberately differs from its source's) it is
# the only way to find that source at all.
MIGRATED_FROM = re.compile(r"Migrated from tests/(browser_[A-Za-z0-9_]+\.rs)")


def _migrated_from(case_text):
    m = MIGRATED_FROM.search(case_text)
    return m.group(1) if m else None


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
        if end > len(lines) or end < first:
            out.append(f"{stem}: retention-header citation :{first}"
                       f"{'-' + str(end) if end != first else ''} is not a resolvable "
                       f"range ({'inverted' if end < first else f'past end of the source, {len(lines)} lines'})")
            continue
        # The nearest backticked, non-citation token, which must be ADJACENT.
        # The window is generous (the family's longest CITED snippet is 137
        # chars and can sit on the previous header line -- see `SNIPPET_MAX`
        # for the derivation) but the token must END within 30
        # chars of the citation. Without the adjacency bound a long window
        # happily binds a citation to an unrelated fn name two sentences back
        # and reports a false pass -- which is worse than reporting nothing.
        # The lookbehind window is SNIPPET_MAX + the 40-char gap `CITE` allows
        # between a snippet and its citation, so this arm can see exactly the
        # snippets the case-file arm can and no more.
        window = body[max(0, m.start() - (SNIPPET_MAX + 40)):m.start()]
        # ITEM 3.2 (batch 7): this bound was `{2,120}`, left behind when batch
        # 6B raised `CITE` to 200, and its own comment ("a fn name in this
        # corpus can be 100 chars") is disproved by the finding that forced the
        # 200 -- the family's longest CITED snippet is 137 and its longest
        # `#[test]` fn name is 226. It failed CLOSED, so it was
        # correctness-preserving: an over-long token yielded "no adjacent
        # backticked construct", a reported problem, never a false pass. It is
        # now the same derived constant as `CITE`'s, so the two cannot drift
        # apart again. Measured across the full sweep before and after: no
        # change in reported problems.
        cands = [mm for mm in re.finditer(r"`([^`\n]{2,%d})`" % SNIPPET_MAX, window)
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
            needles = _needles(raw_tok, _source_items("\n".join(lines)))
            if not needles and re.fullmatch(r"[A-Za-z_][\w]{3,}", raw_tok):
                needles = [raw_tok]
        if not needles:
            out.append(f"{stem}: retention-header citation :{first} has no adjacent "
                       f"backticked construct to resolve against -- reword so the gate "
                       f"can check it (ruling 11 exempts :N only because it is gated)")
            continue
        stmt = "\n".join(_statement(lines, first, end))
        for tok in needles:
            if not _needle_found(tok, stmt):
                out.append(f"{stem}: retention-header citation :{first}"
                           f"{'-' + str(end) if end != first else ''} does not contain "
                           f"{tok!r}")
    return out


def _cite_arm(stem, origin, body, lines, submodules=None):
    """`submodules`: {basename: [line, ...]} for a `#[path]` carrier's siblings.

    A citation naming a file not in that map is a hard failure, not a skip: the
    author pointed at something, and a pointer nobody can resolve is the figure
    in disguise ruling 11 forbids.
    """
    out = []
    submodules = submodules or {}
    qualified = {}
    for m in SUBMOD_CITE.finditer(body):
        snippet, name, first = m.group(1), m.group(2), int(m.group(3))
        end = int(m.group(4)) if m.group(4) else first
        qualified[m.start()] = True
        if name not in submodules:
            out.append(f"{stem}: {origin} citation {name}:{first} for "
                       f"`{snippet[:40]}` names a file that is not a submodule of "
                       f"this target (known: {sorted(submodules) or 'none'})")
            continue
        sub_lines = submodules[name]
        if end > len(sub_lines) or end < first:
            out.append(f"{stem}: {origin} citation {name}:{first}"
                       f"{'-' + str(end) if end != first else ''} for `{snippet[:40]}` "
                       f"is not a resolvable range in {name} "
                       f"({'inverted' if end < first else f'past end, {len(sub_lines)} lines'})")
            continue
        needles = _needles(snippet, _source_items("\n".join(sub_lines)))
        if not needles:
            _NO_NEEDLE[stem] += 1
            continue
        window = "\n".join(_statement(sub_lines, first, end))
        for tok in needles:
            if not _needle_found(tok, window):
                out.append(f"{stem}: {origin} citation {name}:{first}"
                           f"{'-' + str(end) if end != first else ''} does not contain "
                           f"{tok!r} (from `{snippet[:50]}`)")
    for m in CITE.finditer(body):
        if m.start() in qualified:
            continue
        snippet, first, last = m.group(1), int(m.group(2)), m.group(3)
        end = int(last) if last else first
        # RANGE CHECK FIRST (fix round 1, finding I2). This used to sit AFTER the
        # needle gate, so a snippet the gate declined to resolve also escaped the
        # past-end-of-file check -- a citation could point at `:9999` in a
        # 300-line file and be reported as nothing at all. Whether a citation is
        # in range is knowable without any needle, so it is checked unconditionally.
        if end > len(lines) or end < first:
            # `end < first` is M1 (batch 7 fix round 1). The guard used to test
            # only `end > len(lines)`, so an INVERTED range -- `:99212-215`, the
            # shape a mis-typed `:99` next to `212-215` produces -- passed it and
            # then indexed `depth_before[first - 1]` far past the end of the
            # list. That raised `IndexError` out of `_statement`, which is loud
            # but ABORTS THE SWEEP PROCESS, leaving every later stem unchecked --
            # a crash is not a gate result. It is reported as a citation problem
            # instead, which is what it is.
            out.append(f"{stem}: {origin} citation :{first}"
                       f"{'-' + str(end) if end != first else ''} for "
                       f"`{snippet[:40]}` is not a resolvable range "
                       f"({'inverted' if end < first else f'past end of the source, {len(lines)} lines'})")
            continue
        needles = _needles(snippet, _source_items("\n".join(lines)))
        if not needles:
            _NO_NEEDLE[stem] += 1
            continue
        window = "\n".join(_statement(lines, first, end))
        for tok in needles:
            if not _needle_found(tok, window):
                out.append(f"{stem}: {origin} citation :{first}"
                           f"{'-' + str(end) if end != first else ''} does not contain "
                           f"{tok!r} (from `{snippet[:50]}`)")
    return out


# The file the `.all`/`.any` blind spot was demonstrated on. It is a whole-file
# design-spec 5.11 retention, so it stays in the tree after batch 8's family-wide
# deletion of the migrated `.rs` files -- which is what makes it usable as a
# fixture. Its LINE NUMBERS are never written down here: they are searched for at
# run time, because a hardcoded 135/139 is precisely the moving figure ruling 11
# forbids, and an edit to that file's own `//!` header moves both.
_SELFTEST_RS = os.path.join(TESTS, "browser_math_pow_optional_chain_harness.rs")
_SELFTEST_SNIPPET = "errors.iter().all(...)"


def selftest():
    """Mutation kill for the `.all`/`.any` needle blind spot (batch 6A).

    The defect: `_needles` dropped method names shorter than 4 characters, so
    `errors.iter().all(...)` and `errors.iter().any(...)` both reduced to
    ['errors', 'iter'] and a citation onto either statement resolved against the
    other. This asserts, against the real shipped file:

      1. the short method name is IN the needle set (the regression itself);
      2. a citation on the `.all` statement resolves (no false red);
      3. the SAME citation moved onto the `.any` statement is reported (the
         mutation kill -- this was silent before the floor was removed);
      4. a citation moved onto the non-JSON `stderr.contains("E5506")` arm is
         reported (this one was already caught; it is here so a future change
         that trades one arm's sensitivity for the other's is visible).
    """
    if not os.path.exists(_SELFTEST_RS):
        print(f"SELFTEST CANNOT RUN: missing {_SELFTEST_RS}")
        return 2
    lines = open(_SELFTEST_RS).read().split("\n")

    def line_of(fragment):
        hits = [n for n, l in enumerate(lines, 1)
                if fragment in l and not l.lstrip().startswith("//")]
        if len(hits) != 1:
            print(f"SELFTEST CANNOT RUN: {fragment!r} occurs {len(hits)} time(s) "
                  f"in {os.path.basename(_SELFTEST_RS)}, expected exactly 1")
            return None
        return hits[0]

    all_line = line_of("errors.iter().all(")
    any_line = line_of("errors.iter().any(")
    stderr_line = line_of('stderr.contains("E5506")')
    if None in (all_line, any_line, stderr_line):
        return 2

    def cite(n):
        return _header_cite_arm("selftest",
                                f"//! quantifier at `{_SELFTEST_SNIPPET}` (`:{n}`).",
                                lines)

    failures = []
    needles = _needles(_SELFTEST_SNIPPET)
    print(f"needles for `{_SELFTEST_SNIPPET}`: {needles}")
    if "all" not in needles:
        failures.append("the short method name 'all' is not a needle -- the "
                        "length floor is back and the blind spot with it")
    if cite(all_line):
        failures.append(f"the correct citation :{all_line} was reported (false red)")
    for label, n in (("`.any` sibling statement", any_line),
                     ("non-JSON `stderr` arm", stderr_line)):
        problems = cite(n)
        print(f"  drift onto the {label} (:{n}): "
              f"{'CAUGHT' if problems else 'SILENT'}")
        if not problems:
            failures.append(f"drift onto the {label} (:{n}) was NOT caught")

    failures += _submodule_selftest()
    failures += _gated_selftest()
    failures += _check_surface_selftest()
    failures += _declares_mods_selftest()
    failures += _residual_tier_selftest()

    if failures:
        print("\nSELFTEST FAILED")
        for f in failures:
            print(f"  {f}")
        return 1
    print("\nSELFTEST OK — the `.all` -> `.any` citation move is caught, the "
          "correct citation is not, and a `<submodule>.rs:N` citation resolves "
          "against that submodule rather than against the carrier")
    return 0


# A SYNTHETIC carrier/submodule pair, written into a temp dir by the selftest
# itself. Deliberately not a real tree file: every `#[path]` carrier in
# `crates/kali_cli/tests` is deleted by batch 8, so a selftest anchored on one
# would start skipping (or failing) the moment the migration it guards
# completes -- a gate that retires itself exactly when its subject ships.
# THE TWO FILES ARE LINE-ALIGNED ON PURPOSE (fix round 1, I4). Property 4 below
# -- "a `leaf.rs:N` citation is not resolved against the carrier" -- had NO kill
# power in its first form: the carrier line it probed did not contain the cited
# snippet either, so the right and the wrong resolution both produced a problem
# and the assertion (`bool(problems)`) could not tell them apart. A mutation
# resolving qualified citations against the carrier whenever the line was in
# range left `--selftest` fully green.
#
# So the carrier now carries `stderr.contains("E5506")` at a line whose LEAF
# counterpart does not, and the probe cites that line number. Correct behaviour
# reports a problem (leaf's line does not contain the snippet); the mutation
# reports none (the carrier's does). The line numbers are never written down --
# they are searched for at run time, for the reason `_SELFTEST_RS` gives.
_SUBMOD_SELFTEST_CARRIER = '''use std::fs;

fn helper_that_lives_in_the_carrier(source: &str) {
    assert!(source.contains("literal array"), "carrier claim");
}

fn carrier_only_stderr_claim(stderr: &str) {
    assert!(stderr.contains("E5506"), "carrier stderr claim");
}

#[path = "sub/leaf.rs"]
mod leaf;
'''

_SUBMOD_SELFTEST_LEAF = '''use super::*;

#[test]
fn leaf_asserts_the_first_thing() {
    helper_that_lives_in_the_carrier("literal array");
}

fn leaf_filler_so_the_files_do_not_line_up() {
    let _unrelated = 1;
}

#[test]
fn leaf_asserts_the_second_thing() {
    assert!(stderr.contains("E5506"), "leaf claim");
}
'''


def _submodule_selftest():
    """Mutation kill for the `<submodule>.rs:N` citation arm.

    Four properties, all against a synthetic carrier + `#[path]` submodule:

      1. a correct `leaf.rs:N` citation resolves (no false red);
      2. the SAME citation shifted onto a different statement in `leaf.rs` is
         reported (the drift kill);
      3. a citation naming a file that is not a submodule is reported, rather
         than silently skipped;
      4. `leaf.rs:N` is NOT resolved against the carrier. This is the property
         the bare `CITE` pattern got wrong: it matches `(leaf.rs:5)` too, and
         without `SUBMOD_CITE` taking precedence it would check line 5 of the
         CARRIER. The check is that a citation whose line number is valid in the
         carrier and wrong in the submodule still fails.
    """
    import tempfile

    out = []
    with tempfile.TemporaryDirectory() as d:
        carrier = os.path.join(d, "browser_selftest_carrier.rs")
        subdir = os.path.join(d, "sub")
        os.makedirs(subdir)
        with open(carrier, "w") as f:
            f.write(_SUBMOD_SELFTEST_CARRIER)
        with open(os.path.join(subdir, "leaf.rs"), "w") as f:
            f.write(_SUBMOD_SELFTEST_LEAF)

        leaf_lines = _SUBMOD_SELFTEST_LEAF.split("\n")
        carrier_lines = _SUBMOD_SELFTEST_CARRIER.split("\n")
        subs = {os.path.basename(p): p.read_text().split("\n")
                for p in submodule_paths(carrier)}
        if list(subs) != ["leaf.rs"]:
            return [f"submodule selftest: resolved {list(subs)}, expected ['leaf.rs']"]

        def line_in(lines, fragment):
            hits = [n for n, l in enumerate(lines, 1) if fragment in l]
            return hits[0] if len(hits) == 1 else None

        good = line_in(leaf_lines, 'assert!(stderr.contains("E5506"), "leaf claim")')
        other = line_in(leaf_lines, "helper_that_lives_in_the_carrier(")
        # The DISCRIMINATING line: the carrier contains the cited snippet here,
        # the leaf does not. Asserted, not assumed -- if a future edit lines the
        # two files up again this returns a hard "cannot run" instead of a
        # silently toothless green.
        carrier_hit = line_in(carrier_lines, 'assert!(stderr.contains("E5506"), "carrier')
        if None in (good, other, carrier_hit):
            return ["submodule selftest: could not locate its own fixture lines"]
        if carrier_hit > len(leaf_lines):
            return ["submodule selftest: the discriminating carrier line is past "
                    "the end of leaf.rs, so property 4 cannot discriminate"]
        if 'stderr.contains("E5506")' in leaf_lines[carrier_hit - 1]:
            return ["submodule selftest: leaf.rs line {} also carries the cited "
                    "snippet, so property 4 has no kill power -- re-stagger the "
                    "two fixtures".format(carrier_hit)]

        def cite(n, name="leaf.rs"):
            return _cite_arm("selftest", "case file",
                             f'`stderr.contains("E5506")` ({name}:{n})',
                             carrier_lines, subs)

        checks = [
            ("correct leaf.rs citation", cite(good), False),
            ("drift onto another leaf.rs statement", cite(other), True),
            ("citation naming a non-submodule file", cite(good, "nope.rs"), True),
            ("leaf.rs:N silently resolved against the carrier",
             cite(carrier_hit), True),
        ]
        for label, problems, want_problem in checks:
            got = bool(problems)
            print(f"  submodule arm -- {label}: "
                  f"{'reported' if got else 'clean'}")
            if got != want_problem:
                out.append(
                    f"submodule arm: {label} was "
                    f"{'reported' if got else 'NOT reported'}, expected the "
                    f"{'opposite' if want_problem else 'opposite'}")
    return out


def _residual_tier_selftest():
    """Batch 7 fix round 1: the bare-identifier tier, the NO-NEEDLE counter, and
    the inverted-range guard (M1).

    Each property is probed against a synthetic source held here, so the probes
    keep their kill power after the corpus changes -- and each poisoned probe has
    a CONTROL that differs from it only in the poison, so a probe cannot be green
    because it failed to run.
    """
    src = "\n".join([
        "fn assert_browser_thing(source: &str) {",              # 1
        '    assert!(source.contains("needle"), "x");',          # 2
        "}",                                                     # 3
        "",                                                      # 4
        "fn other_helper() {",                                   # 5
        "    let expected_stdout = 1;",                          # 6
        "}",                                                     # 7
        "",                                                      # 8
        # N2 (fix round 2): a fn whose name has the cited one as a PREFIX, plus
        # a call site of it. Both are places a prefix-matching needle would
        # happily bind to.
        "fn assert_browser_thing_with_source() {",               # 9
        "    assert_browser_thing_with_source();",               # 10
        "}",                                                     # 11
        "",                                                      # 12
        '    cmd.env(kali_runtime_contract::HARNESS_ENV, "node");',  # 13
        '    if command == "test" {',                             # 14
    ])
    lines = src.split("\n")
    items = _source_items(src)
    out = []

    # 1. The bare-identifier tier: a source ITEM is a needle; a source LOCAL, a
    #    case-format key, and a word absent from the source are not.
    cases = [
        ("a source item name", "assert_browser_thing", ["assert_browser_thing"]),
        ("a source LOCAL, not an item", "expected_stdout", []),
        ("a case-format key", "stdout_contains", []),
        ("a word absent from the source", "kali", []),
        ("shorter than the four-char floor", "fnx", []),
    ]
    for label, snippet, want in cases:
        got = _needles(snippet, items)
        print(f"  residual tier -- bare `{snippet}` ({label}): {got}")
        if got != want:
            out.append(f"residual tier: `{snippet}` gave {got}, expected {want}")
    # The tier must be OFF when no source is in hand, rather than guessing.
    if _needles("assert_browser_thing", None):
        out.append("residual tier: a bare identifier resolved with no source in "
                   "hand -- the tier must be skipped, not guessed")

    # 2. Kill power: the tier reports a citation that names an item NOT at the
    #    cited line, and does not report the one that is. This is the M8 shape.
    good = _cite_arm("selftest", "case file",
                     "`assert_browser_thing` (:1)", lines)
    bad = _cite_arm("selftest", "case file",
                    "`assert_browser_thing` (:5)", lines)
    print(f"  residual tier -- correct item citation: "
          f"{'reported' if good else 'clean'}; drifted onto another fn: "
          f"{'CAUGHT' if bad else 'SILENT'}")
    if good:
        out.append("residual tier: the correct item citation was reported (false red)")
    if not bad:
        out.append("residual tier: a bare-item citation drifted onto another fn "
                   "was NOT caught -- the tier has no kill power")

    # 2b. N2: WORD BOUNDARIES. A cited fn name that is a PREFIX of a sibling's
    #     must not bind to the sibling -- not to its declaration and not to a
    #     call of it. This is the M8 shape living inside the tier added to catch
    #     it, and it turns a declared-silent citation into an
    #     apparently-resolved one, which is strictly worse.
    for label, line, want in (("its own declaration", 1, False),
                              ("the PREFIX-SIBLING's declaration", 9, True),
                              ("a call of the prefix sibling", 10, True)):
        got = bool(_cite_arm("selftest", "case file",
                             f"`assert_browser_thing` (:{line})", lines))
        print(f"  residual tier -- word boundary, {label} (:{line}): "
              f"{'reported' if got else 'clean'}")
        if got != want:
            out.append(f"word boundary: `assert_browser_thing` at :{line} was "
                       f"{'reported' if got else 'NOT reported'}, expected the opposite")

    # 2c. The two classes fix round 2 moved OUT of the declared tier.
    for label, snippet, line, want_needles in (
            ("a `::` path expression", "kali_runtime_contract::HARNESS_ENV", 13,
             ["HARNESS_ENV"]),
            ("a snippet carrying a quoted literal", 'if command == "test"', 14,
             ["command", "test"])):
        got = _needles(snippet, items)
        print(f"  residual tier -- {label}: {got}")
        if sorted(got) != sorted(want_needles):
            out.append(f"{label}: gave {got}, expected {want_needles}")
        if _cite_arm("selftest", "case file", f"`{snippet}` (:{line})", lines):
            out.append(f"{label}: its correct citation was reported (false red)")
        if not _cite_arm("selftest", "case file", f"`{snippet}` (:2)", lines):
            out.append(f"{label}: a citation drifted onto an unrelated line was "
                       "NOT caught")

    # 3. The NO-NEEDLE counter actually counts. A prose snippet is matched by
    #    `CITE` and yields nothing; the tier must register that rather than
    #    letting it pass unremarked.
    before = _NO_NEEDLE["selftest_nn"]
    _cite_arm("selftest_nn", "case file", "`stdout_contains` (:1)", lines)
    after = _NO_NEEDLE["selftest_nn"]
    print(f"  residual tier -- NO-NEEDLE counter: {before} -> {after}")
    if after != before + 1:
        out.append("residual tier: a needle-less citation did not increment the "
                   "NO-NEEDLE counter, so the declared tier cannot track it")
    _NO_NEEDLE.pop("selftest_nn", None)

    # 4. M1: an INVERTED range is a reported problem, not an `IndexError` that
    #    aborts the sweep and leaves every later stem unchecked.
    try:
        inverted = _cite_arm("selftest", "case file",
                             "`assert_browser_thing` (:99212-215)", lines)
        crashed = False
    except Exception as exc:                                  # noqa: BLE001
        inverted, crashed = [], True
        print(f"  residual tier -- inverted range raised {type(exc).__name__}")
    print(f"  residual tier -- inverted range `:99212-215`: "
          f"{'reported' if inverted else 'SILENT'}")
    if crashed or not inverted:
        out.append("residual tier: an inverted range was not reported as a "
                   "citation problem (M1)")
    return out


def _declares_mods_selftest():
    """Item 3.3: a PLAIN-`mod` carrier is recognised as declaring submodules.

    The regression: `declares_mods` was `"#[path" in <source>`, so a carrier
    using `mod leaf;` instead of `#[path = "..."] mod leaf;` was judged to
    declare nothing. When its submodule then failed to resolve, `check()` fell
    past the single loud "cannot check qualified citations" problem and let the
    per-citation arm emit N misleading "names a file that is not a submodule"
    problems -- one per citation, all pointing the reader at the wrong defect.

    Asserted against the real tree file that has the shape, and against the
    `#[path]` form, and against a file with neither -- so the probe cannot pass
    by returning True for everything.
    """
    out = []
    probes = [
        ("plain `mod x;` (browser_cdp_smoke.rs)", "browser_cdp_smoke.rs", True),
        ("`#[path]` (browser_non_literal_iterator_sources.rs)",
         "browser_non_literal_iterator_sources.rs", True),
        ("no submodules (browser_math_pow_optional_chain_harness.rs)",
         "browser_math_pow_optional_chain_harness.rs", False),
    ]
    for label, name, want in probes:
        path = os.path.join(TESTS, name)
        if not os.path.exists(path):
            out.append(f"declares-mods selftest: missing fixture {name}")
            continue
        with open(path) as fh:
            got = declares_submodules(fh.read())
        print(f"  declares-mods -- {label}: {got}")
        if got != want:
            out.append(f"declares-mods: {label} returned {got}, expected {want}")
        # The superseded predicate, run beside it so the regression stays
        # visible: it is what returns the WRONG answer for the plain-`mod` case.
        with open(path) as fh:
            old = "#[path" in fh.read()
        if want and not old and got:
            print(f"    (superseded `'#[path' in source` predicate: {old} "
                  "-- this is the case it got wrong)")
    return out


def _gated_selftest():
    """Kill power for `_gated_arm`, in both directions and on both surfaces.

    The property that matters is the one `gen_batch6b.py`'s count comparison
    could not state: a citation the readers do not match must be REPORTED, and a
    citation they do match must not be. Each probe below is a string this arm is
    run over directly, so the harness cannot be green because it failed to run --
    the control probes assert a CLEAN result on text that differs from a poisoned
    probe only in the poison.

    THE BLIND-SPOT PROBE is the one labelled "the family-wide shape the count
    comparison misses": it is the exact prose shape (`schemaVersion (:68)`) that
    55 shipped case files carried, and the count comparison passes on it whenever
    some other backtick in the same file accidentally supplies a match. It is
    identified by LABEL below rather than by ordinal -- fix round 2 shrank this
    list from seven probes to six and the prose kept saying "probe 5" for what
    had become probe 4, which is a citation into a list, drifting, in the file
    whose subject is citations drifting (minor 3, fix round 3).
    """
    probes = [
        # (label, body, expect_problem). CASE-FILE SURFACE ONLY -- fix round 2,
        # M6. Two of these probes used to be labelled "retention header" and to
        # pass `extra_readers=(BARE_CITE,)`, and both returned the same result
        # with that argument removed, because `WRITTEN_CITE` cannot produce a hit
        # `BARE_CITE` covers. They demonstrated a reader that was never consulted.
        # Header gatedness is `_header_cite_arm`'s job and is probed there.
        ("a gated citation",
         '# the `json["schemaVersion"]` pin (:68) is exact', False),
        ("a BARE-PROSE citation",
         "# JSON envelope: schemaVersion (:68), command (:69)", True),
        ("an un-backticked citation in a `#` comment",
         "# the blocking helper at (:171) is unreadable", True),
        ("the family-wide shape the count comparison misses",
         "# `ext` cell, the mapping stays 1:1 per trial; envelope: schemaVersion (:68)",
         True),
        ("a ratio that is not a citation (`1:1`)",
         "# the assertion mapping stays 1:1 per trial", False),
        ("a JSON literal in a fixture body is not a citation",
         '"app.js" = """const o = {"schemaVersion":1};"""', False),
        # THE RETENTION-HEADER SURFACE (restored in fix round 3, N4). These two
        # probes were deleted with the dead `extra_readers` argument, and the
        # coverage went with them: `_header_cite_arm` iterates `BARE_CITE` only,
        # so an UN-BACKTICKED `:N` in a `//!` header is invisible to it and this
        # arm is the only thing that sees it. Both were silent at HEAD until the
        # call was put back.
        ("bare prose in a `//!` retention header",
         "//! JSON envelope: schemaVersion (:8), command (:9)", True),
        ("an un-backticked citation in a `//!` retention header",
         "//! the blocking helper at (:8) is unreadable", True),
        ("a BACKTICKED header citation is `_header_cite_arm`'s, not this arm's",
         "//! `kali_bin` (`:8`) is the blocking helper", False),
    ]
    out = []
    for label, body, want in probes:
        got = bool(_gated_arm("selftest", "case file", body))
        print(f"  gated arm -- {label}: {'reported' if got else 'clean'}")
        if got != want:
            out.append(f"gated arm: {label} was {'reported' if got else 'NOT reported'}, "
                       f"expected {'a report' if want else 'clean'}")
    # The blind-spot probe is only meaningful if the count comparison really is
    # blind to it; assert that, rather than asserting it in prose. The probe is
    # found BY LABEL, so re-ordering or adding probes cannot silently point this
    # at a different one. The harness prints the two counts it compares, and they
    # are 1 and 1: the sole `CITE` match is the spurious ``ext` ... 1:1` one,
    # while the real `(:68)` citation goes unread. (The comment here used to
    # claim 3 and 3, contradicting the harness's own printed output.)
    blind = next(body for label, body, _ in probes
                 if label == "the family-wide shape the count comparison misses")
    written = len(re.findall(r"\((?:[A-Za-z0-9_]+\.rs)?:\d+", blind))
    matched = len({m.start() for m in CITE.finditer(blind)}
                  | {m.start() for m in SUBMOD_CITE.finditer(blind)})
    print(f"  gated arm -- count comparison on the blind-spot probe: written={written} "
          f"matched={matched} ({'BLIND' if written == matched else 'would catch'})")
    if written != matched:
        out.append("gated arm: the blind-spot probe no longer demonstrates the "
                   "count comparison's blind spot -- re-pick the probe or drop "
                   "the claim")
    return out


# THE `check()`-SURFACE PROBE FILES (batch 7 fix round 4, I-3).
#
# Line numbers matter and are asserted, so CONTROL and POISON are the same shape
# with line 3 swapped: the poison replaces one filler `//!` line rather than
# adding one, which is what lets both probes cite `:6` and differ ONLY in the
# poison. `helper_named_here` is declared at line 6 of both.
_CHECK_SURFACE_STEM = "selftest_check_surface_probe"
_CHECK_SURFACE_FILLER = "//! Filler; the poison probe replaces this line."
_CHECK_SURFACE_POISON = "//! JSON envelope: schemaVersion (:6), command (:6)"
_CHECK_SURFACE_RS = """\
//! Selftest probe for `check()`'s retention-header surface -- written into
//! `crates/kali_cli/tests` by `--selftest` and deleted again. `helper_named_here` (`:6`)
{filler}

#[allow(dead_code)]
fn helper_named_here(source: &str) {{
    assert!(source.contains("selftest probe"), "probe");
}}
"""
# Deliberately citation-free and backtick-free: the case-file arms must
# contribute nothing, so every problem the paired probe reports comes from the
# retention-header surface under test.
_CHECK_SURFACE_TOML = """\
# Selftest probe written and deleted by --selftest.
name = "selftest_check_surface_probe"
"""


def _check_surface_selftest():
    """I-3: does `check()` REACH `_gated_arm` on the retention-header surface?

    The three probes `_gated_selftest` added in fix round 3 call `_gated_arm`
    DIRECTLY. They prove the arm behaves; they cannot prove `check()` calls it.
    That is exactly the gap round 2 fell through -- it deleted the two
    retention-header invocations, `--selftest` stayed OK and the sweep stayed at
    `EXIT=0`, and the coverage was gone for a whole round. Restoring the calls
    without a probe on `check()` itself leaves the identical regression
    available: delete both call sites again and nothing in the tree fails.

    So this probe drives the real `check()` over a real (temporary) tree file,
    on BOTH surfaces that call the arm:

      * the WHOLE-FILE RETENTION branch (`.rs` with a `//!` header, no case
        file);
      * the PAIRED branch (case file + `//!` header), which is a separate call
        site and was separately deleted.

    Each is run twice against text differing only in the poison, and the poison
    run asserts on the REPORT -- two problems, both naming the retention-header
    surface and both the UNGATED message -- rather than on a bare boolean. An
    un-backticked `:N` is invisible to `_header_cite_arm` (it iterates
    `BARE_CITE`), so `_gated_arm` is the only thing that can produce it, and no
    other arm can make this probe green.

    THE PROBE FILES ARE WRITTEN INTO THE REAL TREE, because `check()` resolves
    both paths itself (`TESTS/browser_<stem>.rs`, `CASES/<stem>.toml`) and a
    temp dir cannot be reached through it -- driving the real call site is the
    whole point. They exist for the duration of four `check()` calls and are
    removed in a `finally`; the `.rs` carries no `#[test]` and the `.toml` no
    case, so a concurrent `cargo test` in that window sees an empty target
    rather than a failing one. A leftover file from a killed run is refused
    rather than overwritten.
    """
    out = []
    rs_path = os.path.join(TESTS, f"browser_{_CHECK_SURFACE_STEM}.rs")
    toml_path = os.path.join(CASES, f"{_CHECK_SURFACE_STEM}.toml")
    for path in (rs_path, toml_path):
        if os.path.exists(path):
            return [f"check() surface: {path} already exists -- refusing to "
                    "overwrite a tree file"]

    def run(filler, paired):
        with open(rs_path, "w") as fh:
            fh.write(_CHECK_SURFACE_RS.format(filler=filler))
        if paired:
            with open(toml_path, "w") as fh:
                fh.write(_CHECK_SURFACE_TOML)
        elif os.path.exists(toml_path):
            os.unlink(toml_path)
        with open(os.devnull, "w") as devnull:
            saved, sys.stdout = sys.stdout, devnull
            try:
                return check(_CHECK_SURFACE_STEM, citations_only=True)
            finally:
                sys.stdout = saved

    try:
        for surface, paired in (("whole-file retention", False),
                                ("paired case file + header", True)):
            clean = run(_CHECK_SURFACE_FILLER, paired)
            poisoned = run(_CHECK_SURFACE_POISON, paired)
            gated = [p for p in poisoned
                     if "retention header citation" in p and "UNGATED" in p]
            print(f"  check() surface -- {surface}: control "
                  f"{len(clean)} problem(s), poisoned {len(poisoned)} "
                  f"({len(gated)} UNGATED)")
            if clean:
                out.append(f"check() surface ({surface}): the control reported "
                           f"{clean} -- the probe is red for a reason other than "
                           "the poison, so its kill power is not attributable")
            if len(gated) != 2:
                out.append(
                    f"check() surface ({surface}): the two un-backticked `:N` in "
                    f"the `//!` header produced {len(gated)} UNGATED report(s), "
                    "expected 2 -- `check()` is not reaching `_gated_arm` on this "
                    "surface (this is the N4 regression, and it is silent by "
                    "definition without this probe)")
            if len(poisoned) != len(gated):
                out.append(
                    f"check() surface ({surface}): {len(poisoned) - len(gated)} "
                    "problem(s) came from somewhere other than the gatedness arm; "
                    "the probe files are supposed to be clean but for the poison")
    finally:
        for path in (rs_path, toml_path):
            if os.path.exists(path):
                os.unlink(path)
        _NO_NEEDLE.pop(_CHECK_SURFACE_STEM, None)
    return out


def main(argv):
    if "--selftest" in argv:
        return selftest()
    if len(argv) < 2:
        print(__doc__)
        return 2
    citations_only = "--citations-only" in argv
    all_problems = []
    stems = [a for a in argv[1:] if not a.startswith("--")]
    for stem in stems:
        all_problems += check(stem, citations_only)
    # STALE RED-LIST ENTRIES ARE A PROBLEM, not a convenience. A red-list that
    # keeps entries nothing fires is exactly the unread pointer this arm exists
    # to catch, one level up -- and it is how a carve-out written for one file
    # silently starts covering a different one. Only checked for stems this run
    # actually visited, so a single-stem invocation does not report the rest.
    visited = {s.partition("=")[0] for s in stems}
    # THE NO-NEEDLE TIER, CHECKED AGAINST ITS DECLARATION (I1). Equality, not a
    # ceiling -- see `NO_NEEDLE_DECLARED`.
    for stem in sorted(visited):
        got, want = _NO_NEEDLE.get(stem, 0), NO_NEEDLE_DECLARED.get(stem, 0)
        if got == want:
            continue
        all_problems.append(
            f"{stem}: {got} citation(s) matched but yielding NO NEEDLE, "
            f"NO_NEEDLE_DECLARED says {want}. "
            + ("A citation that nothing searches for reports clean whether it is "
               "right or wrong; reword it so it names a construct, or raise the "
               "declaration and say why in the report."
               if got > want else
               "The tier shrank -- lower the declaration so it keeps tracking the "
               "corpus rather than sitting above it."))
    for key in sorted(UNGATED_REDLIST):
        if key[0] in visited and key not in _REDLIST_HIT:
            all_problems.append(
                f"{key[0]}: UNGATED_REDLIST entry {key[1]} {key[2]} is STALE -- "
                "nothing fired it. Delete the entry (the citation was reworded or "
                "removed) rather than leaving a carve-out nobody re-checks.")
    if all_problems:
        print("\nCROSSCHECK FAILED")
        for p in all_problems:
            print(f"  {p}")
        return 1
    print("\nCROSSCHECK OK — header structure consistent, every code citation the "
          "gate can resolve resolves, and every one it cannot is declared "
          "(UNGATED_REDLIST / NO_NEEDLE_DECLARED)")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
