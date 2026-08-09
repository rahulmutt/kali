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

    python3 tools/task-18-browser-pilot/citation_tiers.py            # all of it
    python3 tools/task-18-browser-pilot/citation_tiers.py --declare  # the dict
    python3 tools/task-18-browser-pilot/citation_tiers.py --variants # the triage

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
    """The spec list `citation_sweep.sh` builds, in the same order."""
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
