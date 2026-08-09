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
        blob = tempfile.NamedTemporaryFile("w", suffix=".rs", delete=False)
        blob.write(subprocess.run(
            ["git", "-C", X.REPO, "show",
             f"{ref.group(1)}:crates/kali_cli/tests/browser_{stem}.rs"],
            capture_output=True, text=True).stdout)
        blob.close()
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

    total = with_needles = dependent = 0
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
    return total, with_needles, dependent


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
        total, with_needles, dependent = fallback_dependence(specs)
        print("--- NEEDLE FALLBACK DEPENDENCE (resolving specs only) ---")
        print(f"  citation matches                       {total:>5}")
        print(f"  carrying needles                       {with_needles:>5}")
        print(f"  depending on a batch-7 fallback        {dependent:>5}\n")

    if want & {"--all", "--gains"}:
        shipped, no_items = tier_gains(specs)
        print("--- BATCH-7 TIER GAINS (NO-NEEDLE total with a tier switched off) ---")
        print(f"  shipped                                {shipped:>5}")
        print(f"  without the source-defined-item tier   {no_items:>5}"
              f"   (that tier gives needles to {no_items - shipped})")
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
    sys.exit(main(sys.argv))
