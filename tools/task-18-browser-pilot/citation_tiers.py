#!/usr/bin/env python3
"""Every figure the citation gate is described by, regenerated from the tree.

WHY THIS IS COMMITTED (Task 18 batch 7, fix round 2, N1). Batch 7 recorded its
tier figures with commands pointing at `scratchpad/variants.py` and
`scratchpad/measure_nn.py`, which are not in the tree -- and
`NO_NEEDLE_DECLARED`'s own comment said "regenerate with the command recorded
beside the figures in the report", which resolved to a file nobody else has. A
command nobody can run is the same defect as a number nobody can reproduce,
which is the defect this whole gate exists to prevent (ruling 13). Two of the
three figures recorded that way were also simply wrong, and stayed wrong for a
round because nothing re-ran them.

So: one committed instrument, one population, and each section prints the
command-line that produces it.

    python3 tools/task-18-browser-pilot/citation_tiers.py             # all of it
    python3 tools/task-18-browser-pilot/citation_tiers.py --declare   # the dict
    python3 tools/task-18-browser-pilot/citation_tiers.py --variants  # the triage
    python3 tools/task-18-browser-pilot/citation_tiers.py --bare-rule # I-1 / I-2
    python3 tools/task-18-browser-pilot/citation_tiers.py --base=REF  # a git ref

AND THE FIGURES IT PRINTS MUST NOT BE COPIED ANYWHERE (fix round 4). Round 3
built the `--tiers`/`--fallbacks`/`--gains` sections precisely so figures would
be regenerated, and then pasted their output into `batch5_crosscheck.py`'s
comments -- where round 3's own tier move made two of the blocks stale by
exactly +-174 within the same commit. A comment saying "run
`citation_tiers.py --fallbacks`" cannot go stale; a comment saying
"3551 / 3163 / 448" goes stale the next time anyone moves a citation. The only
figures that may sit inline are the ones an equality check corrects on the next
run -- `NO_NEEDLE_DECLARED`, and `BARE_NEEDLE_ADMITTED` through it.

THE POPULATION, stated once because getting it wrong is what hid two defects.
`sweep_specs()` builds exactly the spec list `citation_sweep.sh` passes to
`batch5_crosscheck.py`, and its printed stem count must equal that script's
`sweep over N stems` banner. Specs come in three shapes and the figures below
NEVER average across them:

  * RESOLVING  -- a case file plus a source the gate resolves against (its own
    `browser_<stem>.rs`, a `PRE-TRIM REF:` blob for a U4 trim, or the source a
    U2 split names in its own `Migrated from` line);
  * SOURCELESS -- a case file whose `.rs` was deleted post-migration. Only the
    gatedness arm can run; nothing is resolvable;
  * RETENTION  -- a `//!` header with no case file.
"""

import collections
import glob
import os
import re
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import batch5_crosscheck as X  # noqa: E402

_IDENT = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")

# Pre-trim blobs written by `sweep_specs`, removed on exit. The shell driver
# `rm -rf`s its scratch dir; this used to leak one temp file per U4 trim per
# invocation (minor 6, fix round 3).
_BLOBS = []


def _cleanup_blobs():
    for path in _BLOBS:
        try:
            os.unlink(path)
        except OSError:
            pass
    _BLOBS.clear()


def sweep_specs():
    """The spec SET `citation_sweep.sh` builds.

    ORDER IS NOT THE SAME AND THE DOCSTRING USED TO SAY IT WAS (fix round 4
    minor). Bash's `cases/browser/*.toml` glob collates under the locale, which
    ignores `_`, while `sorted(glob.glob(...))` is bytewise: the two agree on
    every element and disagree on the position of 16 of them, across six stem
    families (`math_round` before vs after `math_round_bracketed_root`, and so
    on). Diff them yourself:

        diff <(cd crates/kali_cli/tests && for t in cases/browser/*.toml; \\
                 do basename "$t" .toml; done) \\
             <(python3 -c 'import glob,os
        for t in sorted(glob.glob("crates/kali_cli/tests/cases/browser/*.toml")):
            print(os.path.basename(t)[:-5])')

    Every figure this instrument prints is an aggregate over the whole list, so
    order is immaterial to all of them; the printed population count is the
    cross-check that the SETS agree. Collapsing the two implementations into one
    is batch 8's (see the report's FR3.7).
    """
    specs = []
    for toml in sorted(glob.glob(os.path.join(X.CASES, "*.toml"))):
        stem = os.path.basename(toml)[:-5]
        rs = os.path.join(X.TESTS, f"browser_{stem}.rs")
        if not os.path.exists(rs):
            named = X.MIGRATED_FROM.search(open(toml).read())
            src = os.path.join(X.TESTS, named.group(1)) if named else None
            specs.append(f"{stem}={src}" if src and os.path.exists(src) else stem)
            continue
        ref = re.search(r"PRE-TRIM REF:\s*(\S+)", open(rs).read())
        if not ref:
            specs.append(stem)
            continue
        # HARD-FAIL ON AN UNREADABLE REF (N5, fix round 3). This used to write
        # `subprocess.run(...).stdout` into the blob without checking the return
        # code, so a `PRE-TRIM REF:` naming a SHA that is not in the repository
        # produced an EMPTY pre-trim source and the instrument carried on. The
        # shell driver hard-stops with `cannot read <ref>` and exit 2; this
        # printed `population: 111 spec(s)` and a table reading `resolved 3015 ->
        # 2970`, `bad-range 0 -> 45`, `silent 412 -> 457`. The one instrument
        # whose whole purpose is that figures cannot be wrong was the one that
        # would print silently wrong figures.
        shown = subprocess.run(
            ["git", "-C", X.REPO, "show",
             f"{ref.group(1)}:crates/kali_cli/tests/browser_{stem}.rs"],
            capture_output=True, text=True)
        if shown.returncode:
            sys.exit(f"cannot read {ref.group(1)}:browser_{stem}.rs -- "
                     f"{shown.stderr.strip()}\nEvery figure this instrument prints "
                     "depends on that blob; refusing to print any.")
        blob = tempfile.NamedTemporaryFile("w", suffix=".rs", delete=False)
        blob.write(shown.stdout)
        blob.close()
        _BLOBS.append(blob.name)
        specs.append(f"{stem}={blob.name}")
    for rs in sorted(glob.glob(os.path.join(X.TESTS, "browser_*.rs"))):
        stem = os.path.basename(rs)[len("browser_"):-3]
        if os.path.exists(os.path.join(X.CASES, f"{stem}.toml")):
            continue
        if any(l.startswith("//!") for l in open(rs)):
            specs.append(stem)
    return specs


def base_tiers(ref):
    """The BASE column of the report's 1.1 table, regenerated from a git archive.

    WHAT THIS IS NOT (fix round 4, I-5). Report 1.1 said the BASE column was
    "the same instrument run against a `git archive` of `8a5c355e1d`". That
    cannot be true and never was: this file did not exist at BASE, and neither
    did the machinery it reads -- BASE's `_needles` takes one argument, and BASE
    has no `_source_items`, no `_needle_found`, no `_NO_NEEDLE` and no
    `WRITTEN_CITE`. Three plausible readings of the sentence gave silent = 1085,
    1259 and 1310 against a table saying 1596, so the sentence was unfalsifiable
    in the worst way: every attempt to check it produced a different number and
    none of them was the one printed.

    WHAT IT IS. The four buckets are HEAD's definitions -- they are the unit of
    the table, and the WRITTEN side has to be one pattern on both columns or the
    columns are not comparable. Everything a bucket is decided BY is BASE's:
    BASE's `CITE`/`SUBMOD_CITE` decide what is matched, BASE's `_needles`
    decides what is searched for, BASE's `_statement` decides the window, and
    BASE compared needles by SUBSTRING (`tok not in stmt`), so this does too.
    The corpus is BASE's as well -- the case files before the reword.

    So the honest claim, and the one the report now makes, is "the gate as it
    existed at BASE, over the corpus as it existed at BASE, partitioned into
    HEAD's four buckets" -- and it is a command, so the next person to doubt it
    runs it instead of picking a reading.
    """
    import importlib.util
    import shutil

    work = tempfile.mkdtemp(prefix="citation_tiers_base_")
    try:
        archive = subprocess.run(
            ["git", "-C", X.REPO, "archive", ref, "--",
             "crates/kali_cli/tests", "tools/task-18-browser-pilot"],
            capture_output=True)
        if archive.returncode:
            sys.exit(f"cannot archive {ref} -- {archive.stderr.decode().strip()}")
        subprocess.run(["tar", "-x", "-C", work], input=archive.stdout, check=True)
        mod = os.path.join(work, "tools/task-18-browser-pilot/batch5_crosscheck.py")
        if not os.path.exists(mod):
            sys.exit(f"{ref} has no tools/task-18-browser-pilot/batch5_crosscheck.py")
        spec = importlib.util.spec_from_file_location("base_crosscheck", mod)
        B = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(B)
        B.REPO = work
        B.TESTS = os.path.join(work, "crates/kali_cli/tests")
        B.CASES = os.path.join(B.TESTS, "cases/browser")
        # BASE's `_needles` is single-argument; BASE has no `_needle_found`.
        # Both adaptations are named here rather than assumed, because getting
        # either wrong silently moves the headline figure.
        try:
            B._needles("probe", None)
            needles = B._needles
        except TypeError:
            needles = lambda s, _items=None: B._needles(s)          # noqa: E731
        found = getattr(B, "_needle_found", lambda tok, stmt: tok in stmt)
        items_of = getattr(B, "_source_items", lambda _text: None)

        specs, blobs = [], []
        for toml in sorted(glob.glob(os.path.join(B.CASES, "*.toml"))):
            stem = os.path.basename(toml)[:-5]
            rs = os.path.join(B.TESTS, f"browser_{stem}.rs")
            if not os.path.exists(rs):
                named = X.MIGRATED_FROM.search(open(toml).read())
                src = os.path.join(B.TESTS, named.group(1)) if named else None
                specs.append((stem, src if src and os.path.exists(src) else None))
                continue
            m = re.search(r"PRE-TRIM REF:\s*(\S+)", open(rs).read())
            if not m:
                specs.append((stem, rs))
                continue
            # Resolved against the REAL repository: the archive is a working
            # tree, not a git dir, and the pre-trim ref is a history pointer.
            shown = subprocess.run(
                ["git", "-C", X.REPO, "show",
                 f"{m.group(1)}:crates/kali_cli/tests/browser_{stem}.rs"],
                capture_output=True, text=True)
            if shown.returncode:
                sys.exit(f"cannot read {m.group(1)}:browser_{stem}.rs at {ref}")
            blob = tempfile.NamedTemporaryFile("w", suffix=".rs", delete=False,
                                               dir=work)
            blob.write(shown.stdout)
            blob.close()
            blobs.append(blob.name)
            specs.append((stem, blob.name))
        for rs in sorted(glob.glob(os.path.join(B.TESTS, "browser_*.rs"))):
            stem = os.path.basename(rs)[len("browser_"):-3]
            if os.path.exists(os.path.join(B.CASES, f"{stem}.toml")):
                continue
            if any(l.startswith("//!") for l in open(rs)):
                specs.append((stem, rs))

        b = collections.Counter()
        for stem, source in specs:
            toml = os.path.join(B.CASES, f"{stem}.toml")
            if not os.path.exists(toml):
                continue
            text = open(toml).read()
            lines = (open(source).read().split("\n")
                     if source and os.path.exists(source) else None)
            seen = {}
            for m in list(B.SUBMOD_CITE.finditer(text)) + list(B.CITE.finditer(text)):
                seen.setdefault(m.start(), m)
            kind = {}
            for m in seen.values():
                if lines is None:
                    kind[(m.start(), m.end())] = "no-source"
                    continue
                qualified = m.re is B.SUBMOD_CITE
                first = int(m.group(3) if qualified else m.group(2))
                last = m.group(4) if qualified else m.group(3)
                end = int(last) if last else first
                if qualified:
                    kind[(m.start(), m.end())] = "qualified"
                    continue
                got = needles(m.group(1), items_of("\n".join(lines)))
                if not got:
                    kind[(m.start(), m.end())] = "no-needle"
                elif end > len(lines) or end < first:
                    kind[(m.start(), m.end())] = "bad-range"
                else:
                    stmt = "\n".join(B._statement(lines, first, end))
                    kind[(m.start(), m.end())] = (
                        "RESOLVED" if all(found(t, stmt) for t in got) else "FAILS")
            for w in X.WRITTEN_CITE.finditer(text):
                hits = [k for (a, e), k in kind.items() if a <= w.start() < e]
                if "RESOLVED" in hits:
                    b["resolved against its own source"] += 1
                elif "qualified" in hits:
                    b["resolved against a #[path] submodule"] += 1
                elif hits:
                    b[hits[0]] += 1
                else:
                    b["ungated"] += 1
        return len(specs), b
    finally:
        shutil.rmtree(work, ignore_errors=True)


def _sides(spec):
    """(stem, case-file text or None, source lines or None)."""
    stem, _, override = spec.partition("=")
    toml = os.path.join(X.CASES, f"{stem}.toml")
    text = open(toml).read() if os.path.exists(toml) else None
    rs = override or os.path.join(X.TESTS, f"browser_{stem}.rs")
    lines = open(rs).read().split("\n") if os.path.exists(rs) else None
    return stem, text, lines


def _matches(text):
    """Distinct citation matches, de-duplicated by start offset the way the gate
    does (a qualified citation matches both patterns)."""
    seen = {}
    for m in list(X.SUBMOD_CITE.finditer(text)) + list(X.CITE.finditer(text)):
        seen.setdefault(m.start(), m)
    return seen


def _range_of(m):
    qualified = m.re is X.SUBMOD_CITE
    first = int(m.group(3) if qualified else m.group(2))
    last = m.group(4) if qualified else m.group(3)
    return first, (int(last) if last else first), qualified


def tier_table(specs):
    """The WRITTEN-citation partition: one unit, buckets that sum to the total."""
    b = collections.Counter()
    for spec in specs:
        stem, text, lines = _sides(spec)
        if text is None:
            continue
        matches = _matches(text)
        kind = {}
        for m in matches.values():
            if lines is None:
                kind[(m.start(), m.end())] = "no-source"
                continue
            first, end, qualified = _range_of(m)
            if qualified:
                # Resolved against the submodule, not this source. The sweep
                # reports 0 problems for all of them; classified separately
                # rather than resolved a second time here.
                kind[(m.start(), m.end())] = "qualified"
                continue
            needles = X._needles(m.group(1), X._source_items("\n".join(lines)))
            if not needles:
                kind[(m.start(), m.end())] = "no-needle"
            elif end > len(lines) or end < first:
                kind[(m.start(), m.end())] = "bad-range"
            else:
                stmt = "\n".join(X._statement(lines, first, end))
                kind[(m.start(), m.end())] = (
                    "RESOLVED" if all(X._needle_found(t, stmt) for t in needles)
                    else "FAILS")
        for w in X.WRITTEN_CITE.finditer(text):
            hits = [k for (a, e), k in kind.items() if a <= w.start() < e]
            if "RESOLVED" in hits:
                b["resolved against its own source"] += 1
            elif "qualified" in hits:
                b["resolved against a #[path] submodule"] += 1
            elif hits:
                b[hits[0]] += 1
            else:
                b["ungated"] += 1
    return b


def declaration(specs):
    """`NO_NEEDLE_DECLARED`, regenerated by running the real gate."""
    import contextlib
    import io
    X._NO_NEEDLE.clear()
    with contextlib.redirect_stdout(io.StringIO()):
        for spec in specs:
            X.check(spec, citations_only=True)
    return {k: v for k, v in sorted(X._NO_NEEDLE.items()) if v}


def fallback_dependence(specs):
    """How many resolved citations depend on batch 7's two `_needles` fallbacks.

    Stated over the population it is measured on: every citation match in a
    RESOLVING spec. The figure this replaces (419) was computed over a subset --
    it silently dropped the sourceless and `Migrated from` stems -- and did not
    match either that subset or this population.
    """
    def pre_batch7(snippet):
        s = snippet.strip()
        if X._NAME_ELISION.search(s):
            return []
        lead = re.match(r"([A-Za-z_][A-Za-z0-9_]*)\s*=(?!=)", s)
        if lead and lead.group(1) in X.CASE_KEYS:
            return [l for l in X._SNIPPET_LITERAL.findall(s[lead.end():]) if l]
        tok = X._distinctive(s)
        if not tok:
            return []
        return [tok] + sorted({m for m in X._METHOD.findall(s) if m != tok})

    total = with_needles = dependent = no_distinctive = 0
    for spec in specs:
        _stem, text, lines = _sides(spec)
        if text is None or lines is None:
            continue
        items = X._source_items("\n".join(lines))
        for m in _matches(text).values():
            total += 1
            if X._needles(m.group(1), items):
                with_needles += 1
                if not pre_batch7(m.group(1)):
                    dependent += 1
                # THE SECOND PREDICATE, printed so the two can never be
                # conflated again. "`_distinctive` is None and `_needles` is
                # non-empty" is a DIFFERENT question from "pre-batch-7 `_needles`
                # was empty", and the gap between them is one snippet:
                # `exit = "success"`, 48 citations, which `_distinctive` declines
                # (no `(`, `.` or `[`) but `_needles` never asks it about,
                # because the `CASE_KEYS` branch returns `['success']` two lines
                # earlier. That branch is BATCH 6, so those 48 do not depend on a
                # batch-7 fallback -- which is why the two numbers differ and
                # neither is wrong.
                if not X._distinctive(m.group(1).strip()):
                    no_distinctive += 1
    return total, with_needles, dependent, no_distinctive


def describe_tier(specs):
    """What the DECLARED tier actually contains -- the figures every round so far
    has described in prose and none has regenerated.

    Three rounds in a row the *description* of a correct mechanism was the
    defect: "prose that names no code position at all" (false -- most of it
    occurs verbatim in its own source), then "none is a construct a search can
    pin to one statement" (false -- most of them are). So the composition is
    computed rather than characterised.
    """
    composition = collections.Counter()
    verbatim = resolves = pinned = total = 0
    for spec in specs:
        _stem, text, lines = _sides(spec)
        if text is None or lines is None:
            continue
        items = X._source_items("\n".join(lines))
        code = "\n".join(l for l in lines if not l.lstrip().startswith("//"))
        for m in _matches(text).values():
            if X._needles(m.group(1), items):
                continue
            snippet = m.group(1).strip()
            total += 1
            composition[snippet] += 1
            if snippet in code:
                verbatim += 1
            first, end, qualified = _range_of(m)
            if qualified or first > len(lines) or end > len(lines) or end < first:
                continue
            if snippet in "\n".join(X._statement(lines, first, end)):
                resolves += 1
                # "pins one statement" in the sense the gate uses: both +-1
                # shifts lose it.
                shifts = []
                for delta in (1, -1):
                    a, b = first + delta, end + delta
                    shifts.append(a < 1 or b > len(lines) or snippet not in
                                  "\n".join(X._statement(lines, a, b)))
                if all(shifts):
                    pinned += 1
    return total, verbatim, resolves, pinned, composition


def mutation_comparison(specs):
    """The +-1 kill comparison, and the cost of word-bounding the needles.

    Both were typed into the round-2 report with no command. The first is the
    regression check for N2; the second is the claim that N2's fix was free.
    """
    substring = (lambda tok, stmt: tok in stmt)
    killed = {"substring": 0, "word-bounded": 0}
    mutants = 0
    loosened = 0
    for spec in specs:
        _stem, text, lines = _sides(spec)
        if text is None or lines is None:
            continue
        items = X._source_items("\n".join(lines))
        for m in _matches(text).values():
            needles = X._needles(m.group(1), items)
            if not needles:
                continue
            first, end, qualified = _range_of(m)
            if qualified or end > len(lines) or end < first:
                continue
            stmt = "\n".join(X._statement(lines, first, end))
            # "0 pass by substring and fail word-bounded" -- N2's freeness.
            for tok in needles:
                if substring(tok, stmt) and not X._needle_found(tok, stmt):
                    loosened += 1
            for label, found in (("substring", substring),
                                 ("word-bounded", X._needle_found)):
                if not all(found(tok, stmt) for tok in needles):
                    continue
                for delta in (1, -1):
                    a, b = first + delta, end + delta
                    if label == "substring":
                        mutants += 1
                    if a < 1 or b > len(lines):
                        killed[label] += 1
                        continue
                    shifted = "\n".join(X._statement(lines, a, b))
                    if not all(found(tok, shifted) for tok in needles):
                        killed[label] += 1
    return mutants, killed, loosened


def admissible(specs):
    """Which still-declared snippets could be admitted as their own needle, and
    at what cost -- measured one snippet at a time across the whole sweep.

    This is the measurement behind `BARE_NEEDLE_ADMITTED`, and it is the reason
    that constant is a declaration rather than a rule: it separates the bare
    backticks this corpus uses as CONSTRUCTS from the ones it uses as LABELS,
    and nothing lexical does.
    """
    import contextlib
    import io
    original = X._needles

    def run(fn):
        X._needles = fn
        X._NO_NEEDLE.clear()
        with contextlib.redirect_stdout(io.StringIO()):
            problems = [p for spec in specs
                        for p in X.check(spec, citations_only=True)]
        total = sum(X._NO_NEEDLE.values())
        X._needles = original
        return problems, total

    def admit(name):
        def needles(snippet, source_items=None):
            base = original(snippet, source_items)
            return base if base else ([snippet.strip()]
                                      if snippet.strip() == name else [])
        return needles

    shipped, declared_total = run(original)
    counts = collections.Counter()
    for spec in specs:
        _stem, text, lines = _sides(spec)
        if text is None or lines is None:
            continue
        items = X._source_items("\n".join(lines))
        for m in _matches(text).values():
            if not X._needles(m.group(1), items):
                counts[m.group(1).strip()] += 1
    rows = []
    for snippet, n in counts.most_common():
        problems, _ = run(admit(snippet))
        new = [p for p in problems if p not in shipped]
        rows.append((snippet, n, len(new)))
    return declared_total, rows


class _ItemsWithSource(frozenset):
    """`_source_items`'s return value, carrying the source text it came from.

    Instrument-local. `_needles` receives the item set but never the source, so a
    rule phrased over the SOURCE TEXT -- the one below -- cannot be evaluated
    inside the gate as it stands. Rather than change `_needles`'s signature to
    measure a rule that is not shipped, the instrument hands the text down on the
    object the gate already passes. A frozenset subclass is exactly as good a
    frozenset, so nothing in the gate behaves differently under this patch.
    """
    text = ""


def bare_rule(specs):
    """Every claim `BARE_NEEDLE_ADMITTED`'s justification makes, measured.

    THE POPULATION IS THE TIER AS IT WOULD BE WITHOUT THE DECLARATION, because
    that is the corpus the rejected rule would have been applied to. Measuring
    "admitting every bare identifier" against the SHIPPED gate answers a
    different question: the three snippets the declaration already admits have
    needles there, so the rule's cost on them cannot show up.

    Four rows, and the reason there are four is that fix round 3's own sentence
    ("those produce 211 FALSE reds") averaged across them: 211 is a CITATION
    count taken from `--admissible`'s FALSE RED row and reported as a FAILURE
    count, over a population the sentence did not name. The failure counts are
    70, 50 and 200 depending on which population is meant, and none of them is
    211.
    """
    original = X._needles
    orig_items = X._source_items
    saved_declared = X.BARE_NEEDLE_ADMITTED

    def undeclared(snippet, source_items=None):
        X.BARE_NEEDLE_ADMITTED = frozenset()
        try:
            return original(snippet, source_items)
        finally:
            X.BARE_NEEDLE_ADMITTED = saved_declared

    def items_with_source(text):
        out = _ItemsWithSource(orig_items(text))
        out.text = text
        return out

    import contextlib
    import io

    def run(fn):
        X._needles = fn
        X._NO_NEEDLE.clear()
        with contextlib.redirect_stdout(io.StringIO()):
            problems = [p for spec in specs
                        for p in X.check(spec, citations_only=True)]
        X._needles = original
        return problems

    X._source_items = items_with_source
    try:
        baseline = run(undeclared)

        # The tier's composition, with the declaration off.
        counts = collections.Counter()
        for spec in specs:
            _stem, text, lines = _sides(spec)
            if text is None or lines is None:
                continue
            items = X._source_items("\n".join(lines))
            for m in _matches(text).values():
                if not undeclared(m.group(1), items):
                    counts[m.group(1).strip()] += 1
        bare = {s: n for s, n in counts.items() if X.BARE_IDENT.fullmatch(s)}
        bare4 = {s: n for s, n in bare.items() if len(s) >= 4}

        def admit(names, floor=0):
            def needles(snippet, source_items=None):
                b = undeclared(snippet, source_items)
                if b:
                    return b
                s = snippet.strip()
                return [s] if s in names and len(s) >= floor else []
            return needles

        # THE DERIVABLE RULE (fix round 4, I-2). Fix round 3 wrote "no lexical
        # predicate separates a label from a construct -- the difference is what
        # the author meant the backtick to do". That is false, and the
        # counter-example is the direct analogue of the tier one line above it in
        # `_needles` ("the source defines an item of that name"): the source
        # INDEXES a name of its own by that spelling, `json["<name>"]`. Whether
        # it separates them on this corpus is a measurement, printed below, not
        # a sentence.
        admitted = collections.Counter()

        def json_indexed(snippet, source_items=None):
            b = undeclared(snippet, source_items)
            if b:
                return b
            s = snippet.strip()
            if not X.BARE_IDENT.fullmatch(s) or len(s) < 4:
                return []
            if f'["{s}"]' in getattr(source_items, "text", ""):
                admitted[s] += 1
                return [s]
            return []

        rows = []
        for label, fn in (
                ("admit every bare identifier in the tier", admit(set(bare))),
                ("  the same at the file's own >=4-char floor",
                 admit(set(bare4), 4)),
                ("admit every declared snippet, bare or not", admit(set(counts))),
                ("RULE: the source json-indexes this name", json_indexed)):
            problems = run(fn)
            rows.append((label, len([p for p in problems if p not in baseline])))
    finally:
        X._needles = original
        X._source_items = orig_items
        X.BARE_NEEDLE_ADMITTED = saved_declared

    # Of the citations the declaration admits, how many would a +-1 drift
    # actually be CAUGHT on. "All 174 pass today, so a future drift in ANY of
    # them is now caught" was the round-3 claim; passing is not pinning, and the
    # gate's own standard for pinned is the +-1 shift.
    per, pinned = collections.Counter(), collections.Counter()
    for spec in specs:
        _stem, text, lines = _sides(spec)
        if text is None or lines is None:
            continue
        items = X._source_items("\n".join(lines))
        for m in _matches(text).values():
            s = m.group(1).strip()
            if s not in X.BARE_NEEDLE_ADMITTED or X._needles(m.group(1), items) != [s]:
                continue
            first, end, qualified = _range_of(m)
            if qualified or end > len(lines) or end < first:
                continue
            per[s] += 1
            if all(first + d < 1 or end + d > len(lines) or not X._needle_found(
                    s, "\n".join(X._statement(lines, first + d, end + d)))
                    for d in (1, -1)):
                pinned[s] += 1
    return counts, bare, bare4, rows, dict(admitted), per, pinned


def tier_gains(specs):
    """How many citations each batch-7 `_needles` tier gives a needle to.

    Measured by DISABLING one tier at a time and re-running the real gate, so
    each figure is the tier's own contribution rather than a hand-partition of
    the snippets. `source_items=None` disables exactly the source-defined-item
    tier, which is why it is passed rather than a rewritten `_needles`.
    """
    import contextlib
    import io
    original = X._needles

    def total(fn):
        X._needles = fn
        X._NO_NEEDLE.clear()
        with contextlib.redirect_stdout(io.StringIO()):
            for spec in specs:
                X.check(spec, citations_only=True)
        X._needles = original
        return sum(X._NO_NEEDLE.values())

    shipped = total(original)
    no_items = total(lambda s, items=None: original(s, None))
    return shipped, no_items


def variants(specs):
    """The I1 triage: what each faithful "bare identifier is a needle" costs."""
    import contextlib
    import io
    original = X._needles

    def run():
        X._NO_NEEDLE.clear()
        with contextlib.redirect_stdout(io.StringIO()):
            return [p for spec in specs for p in X.check(spec, citations_only=True)]

    def make(floor, only_when_empty, filter_keys, source_defined):
        def needles(snippet, source_items=None):
            base = original(snippet, source_items)
            if base and only_when_empty:
                return base
            s = snippet.strip()
            if X._NAME_ELISION.search(s) or not _IDENT.fullmatch(s) or len(s) < floor:
                return base
            if filter_keys and s in X.CASE_KEYS:
                return base
            if source_defined and not (source_items and s in source_items):
                return base
            return sorted(set(base) | {s})
        return needles

    shipped = run()
    rows = [("shipped (source-defined item only)", original)]
    rows += [
        ("bare id >=4, only where _needles == []", make(4, True, False, False)),
        ("bare id >=4, added alongside existing", make(4, False, False, False)),
        ("bare id >=3, only where _needles == []", make(3, True, False, False)),
        ("bare id >=4, CASE_KEYS filtered", make(4, True, True, False)),
    ]
    out = []
    for label, fn in rows:
        X._needles = fn
        problems = run()
        X._needles = original
        new = [p for p in problems if p not in shipped]
        out.append((label, len(new), len({p.split(":")[0] for p in new})))
    X._needles = original
    run()                      # leave `_NO_NEEDLE` holding the shipped numbers
    return out


def main(argv):
    base_ref = next((a.split("=", 1)[1] for a in argv[1:]
                     if a.startswith("--base=")), None)
    if base_ref:
        n, b = base_tiers(base_ref)
        print(f"$ python3 tools/task-18-browser-pilot/citation_tiers.py "
              f"--base={base_ref}")
        print(f"--- WRITTEN-CITATION PARTITION at {base_ref} ---")
        print("  the gate AS IT EXISTED AT THAT REF, over the corpus as it existed")
        print("  there, in HEAD's four buckets. NOT 'the same instrument': this "
              "file\n  did not exist at BASE. See `base_tiers`.")
        print("  CALIBRATION: `--base=HEAD` must reproduce `--tiers` exactly. If "
              "it does\n  not, the adapter is the defect and no other reading of "
              "this mode is worth\n  arguing about.")
        print(f"  population: {n} spec(s)")
        for k, v in b.most_common():
            print(f"  {k:<38} {v:>5}")
        print(f"  {'TOTAL written':<38} {sum(b.values()):>5}")
        print(f"  {'silent (never re-resolved)':<38} "
              f"{sum(v for k, v in b.items() if not k.startswith('resolved')):>5}")
        return 0
    specs = sweep_specs()
    kinds = collections.Counter()
    for spec in specs:
        _stem, text, lines = _sides(spec)
        kinds["retention" if text is None else
              ("resolving" if lines is not None else "sourceless")] += 1
    want = set(a for a in argv[1:] if a.startswith("--")) or {"--all"}

    print(f"$ python3 tools/task-18-browser-pilot/citation_tiers.py")
    print(f"population: {len(specs)} spec(s) -- this must equal citation_sweep.sh's "
          f"`sweep over N stems`")
    print(f"  resolving={kinds['resolving']} sourceless={kinds['sourceless']} "
          f"retention={kinds['retention']}\n")

    if want & {"--all", "--variants"}:
        print("--- I1 TRIAGE: cost of each 'bare identifier is a needle' variant ---")
        for label, n, files in variants(specs):
            print(f"  {label:<42} {n:>4} new failure(s) / {files:>3} file(s)")
        print()

    if want & {"--all", "--tiers"}:
        print("--- WRITTEN-CITATION PARTITION (buckets sum to the total) ---")
        b = tier_table(specs)
        for k, v in b.most_common():
            print(f"  {k:<38} {v:>5}")
        silent = sum(v for k, v in b.items() if not k.startswith("resolved"))
        print(f"  {'TOTAL written':<38} {sum(b.values()):>5}")
        print(f"  {'silent (never re-resolved)':<38} {silent:>5}\n")

    if want & {"--all", "--fallbacks"}:
        total, with_needles, dependent, no_distinctive = fallback_dependence(specs)
        print("--- NEEDLE FALLBACK DEPENDENCE (resolving specs only) ---")
        print(f"  citation matches                             {total:>5}")
        print(f"  carrying needles                             {with_needles:>5}")
        print(f"  pre-batch-7 `_needles` was empty             {dependent:>5}"
              f"   <- 'depends on a batch-7 fallback'")
        print(f"  no distinctive leading identifier            {no_distinctive:>5}"
              f"   <- a DIFFERENT predicate; the gap is `exit = \"success\"`,")
        print(f"  {'':>45}      which batch 6's CASE_KEYS branch already read\n")

    if want & {"--all", "--gains"}:
        shipped, no_items = tier_gains(specs)
        print("--- BATCH-7 TIER GAINS (NO-NEEDLE total with a tier switched off) ---")
        print(f"  shipped                                {shipped:>5}")
        print(f"  without the source-defined-item tier   {no_items:>5}"
              f"   (that tier gives needles to {no_items - shipped})")
        print()

    if want & {"--all", "--describe"}:
        total, verbatim, resolves, pinned, composition = describe_tier(specs)
        print("--- WHAT THE DECLARED TIER CONTAINS ---")
        print(f"  citations in the tier                  {total:>5}")
        print(f"  occurring verbatim in their own source {verbatim:>5}")
        print(f"  resolving at their own cited line      {resolves:>5}")
        print(f"  pinned (both +-1 shifts lose them)     {pinned:>5}")
        print(f"  distinct snippets                      {len(composition):>5}")
        keys = sum(n for s, n in composition.items() if s in X.CASE_KEYS)
        print(f"  of which case-format keys (CASE_KEYS)  {keys:>5}")
        for s, n in composition.most_common(8):
            print(f"      {n:4d}  `{s[:58]}`")
        print()

    if want & {"--all", "--mutation"}:
        mutants, killed, loosened = mutation_comparison(specs)
        print("--- +-1 MUTATION COMPARISON (N2 regression check) ---")
        print(f"  shifted citations                      {mutants:>5}")
        print(f"  killed, substring comparison           {killed['substring']:>5}")
        print(f"  killed, word-bounded (shipped)         {killed['word-bounded']:>5}")
        print(f"  needles passing substring but failing word-bounded "
              f"{loosened:>5}   <- N2's cost")
        print()

    if want & {"--all", "--admissible"}:
        declared_total, rows = admissible(specs)
        free = [r for r in rows if r[2] == 0]
        cost = [r for r in rows if r[2]]
        print("--- STILL-DECLARED SNIPPETS: admissible at zero cost? ---")
        print(f"  declared tier total {declared_total}")
        print(f"  ZERO COST : {len(free):>3} snippet(s), "
              f"{sum(n for _, n, _ in free):>4} citation(s)")
        for s, n, _ in free[:10]:
            print(f"      {n:4d}  `{s[:58]}`")
        print(f"  FALSE RED : {len(cost):>3} snippet(s), "
              f"{sum(n for _, n, _ in cost):>4} citation(s)")
        for s, n, f in cost[:10]:
            print(f"      {n:4d} (+{f} new failures)  `{s[:48]}`")
        print()

    if want & {"--all", "--bare-rule"}:
        counts, bare, bare4, rows, admitted, per, pinned = bare_rule(specs)
        print("--- BARE IDENTIFIERS: what admitting them costs, by population ---")
        print("  population: the declared tier with BARE_NEEDLE_ADMITTED OFF")
        print(f"    every declared snippet            {len(counts):>4} snippet(s), "
              f"{sum(counts.values()):>4} citation(s)")
        print(f"    of which bare identifiers         {len(bare):>4} snippet(s), "
              f"{sum(bare.values()):>4} citation(s)")
        print(f"    of those, >= 4 chars              {len(bare4):>4} snippet(s), "
              f"{sum(bare4.values()):>4} citation(s)")
        print("  cost, measured against that baseline -- these are FAILURE counts,")
        print("  and none of them is `--admissible`'s FALSE RED row, which counts")
        print("  CITATIONS:")
        for label, n in rows:
            print(f"    {label:<44} {n:>4} new failure(s)")
        print(f"  the rule admits {dict(sorted(admitted.items(), key=lambda kv: -kv[1]))}"
              f" = {sum(admitted.values())} citation(s)")
        print(f"  BARE_NEEDLE_ADMITTED is {sorted(X.BARE_NEEDLE_ADMITTED)}; the rule "
              f"admits exactly it: {set(admitted) == set(X.BARE_NEEDLE_ADMITTED)}")
        print(f"  it rejects "
              f"{sorted(s for s in bare if s not in admitted)}")
        print("  of the admitted citations, how many a +-1 drift is CAUGHT on:")
        for s in sorted(per, key=lambda k: -per[k]):
            print(f"    {s:<12} {pinned[s]:>4} of {per[s]:>4}")
        print(f"    {'TOTAL':<12} {sum(pinned.values()):>4} of {sum(per.values()):>4}")
        print("  rank of each admitted snippet in the undeclared tier, by size:")
        order = [s for s, _ in counts.most_common()]
        for s in sorted(X.BARE_NEEDLE_ADMITTED, key=lambda k: order.index(k)):
            print(f"    {s:<12} #{order.index(s) + 1} of {len(order)}  "
                  f"({counts[s]} citation(s))")
        print()

    if want & {"--all", "--declare"}:
        d = declaration(specs)
        print("--- NO_NEEDLE_DECLARED (paste into batch5_crosscheck.py) ---")
        print("NO_NEEDLE_DECLARED = {")
        for k, v in d.items():
            print(f'    "{k}": {v},')
        print("}")
        print(f"# total {sum(d.values())} across {len(d)} stems")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main(sys.argv))
    finally:
        _cleanup_blobs()
