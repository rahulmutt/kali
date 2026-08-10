#!/usr/bin/env python3
"""Reword every UNGATED `:N` citation in `cases/browser/` so the gate can read it.

Ruling 11 exempts `:N` code citations from the no-moving-figures rule ONLY
because they are mechanically gated -- "a pointer nothing re-resolves is a figure
in disguise". `batch5_crosscheck.py`'s `_gated_arm` (batch 7) measures that
premise family-wide and it was false for most of the family: a citation written
as bare prose (`schemaVersion (:68)`) matches no reader pattern and is never
resolved, reporting `0 problem(s)` whether it is right or wrong.

This is the reword half. For each ungated site it inserts a backticked construct
immediately before the number, and THE CONSTRUCT IS READ OUT OF THE CITED SOURCE
LINES -- never invented, never taken from the surrounding prose. That matters for
two reasons:

  * it cannot "fix" a citation by changing its number, which this project has
    banned (the number is never touched here; only prose is added beside it);
  * it cannot fix a citation by asserting something the source does not say. The
    snippet is quoted from the very lines the citation points at, so a citation
    that is STALE cannot be reworded at all -- no construct is found, and the
    site is reported for manual re-derivation instead of being papered over.

Argument lists are carried verbatim where that is safe and elided to `(...)`
where it is not; `_elide` states the rule and why. The only thing never carried
is a backslash: a `#` comment and a multi-line TOML string escape it
differently, and the gate reads the file's raw bytes, so a needle taken from
`"0\n"` would be right on one surface and wrong on the other.

Every rewrite is VERIFIED BEFORE IT IS WRITTEN: the candidate snippet is run
through the real `_needles` and the real `_statement` expander against the real
source lines, and is rejected unless it produces a non-empty needle set that
resolves. A snippet the gate would merely MATCH without resolving is not
accepted -- that is the "loosen it until it passes" failure the gate exists to
prevent, one layer down.

Usage: reword_ungated_citations.py [--apply] [STEM ...]
       (default: report only, over every stem)
Exit 0 if every ungated site was reworded (or there were none), 1 otherwise.
"""

import os
import re
import subprocess
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import batch5_crosscheck as X  # noqa: E402

TESTS = X.TESTS
CASES = X.CASES

# The gate's own snippet bound, read from the gate rather than retyped, so the
# reworder can never emit a snippet `CITE` would silently drop. This corpus's
# `#[test]` fn names really do run past 130 characters, and the whole point of
# the 200 bound is that a citation whose snippet exceeds it is INVISIBLE rather
# than unresolved.
_SNIPPET_BOUND = int(re.search(r"\{3,(\d+)\}", X.CITE.pattern).group(1))

# An expression worth quoting: an identifier followed by at least one call,
# field access or index. Deliberately anchored on a word boundary so `!errors`
# yields `errors...` and a macro name (`assert_eq!(`) is not mistaken for a call.
# The optional leading `.` is load-bearing: a rustfmt-split builder chain puts
# `.arg("--max-threads")` on a line of its own, and `_needles` reads a method
# name only through `_METHOD`, which requires the dot. Dropping it turns the
# snippet into `arg(...)`, whose leading identifier is under `_distinctive`'s
# four-character floor, and the citation is then matched but never resolved.
_EXPR = re.compile(
    r"\.?[A-Za-z_][A-Za-z0-9_]*"
    r"(?:::[A-Za-z_][A-Za-z0-9_]*)*"
    r"(?:\.[A-Za-z_][A-Za-z0-9_]*|\[[^\[\]\n]*\]|\((?:[^()\n]*)\))+")

# A rustfmt-SPLIT call or signature: the `(` closes on a later line, so `_EXPR`
# (which needs a balanced pair on one line) sees nothing at all. This corpus's
# longest `#[test]` fn names are exactly the ones rustfmt splits, so without
# this every citation onto such a signature was unanchorable -- and those are
# the citations U3 most wants, since they point at the source `#[test]` fn a
# case was migrated from.
_OPEN_CALL = re.compile(r"[A-Za-z_][A-Za-z0-9_]*\((?=\s*$)")


def _pretrim_lines(stem, toml_text):
    """The source a case file's `:N` citations are numbered against.

    A U4 trim's case-file citations are PRE-TRIM line numbers (ruling 9), so the
    blob, not the working tree, is the right side -- the same rule
    `citation_sweep.sh` applies. A U2 split names its source in its own header.
    """
    rs = os.path.join(TESTS, f"browser_{stem}.rs")
    if not os.path.exists(rs):
        named = X._migrated_from(toml_text)
        if not named:
            return None
        rs = os.path.join(TESTS, named)
        return open(rs).read().split("\n") if os.path.exists(rs) else None
    ref = re.search(r"PRE-TRIM REF:\s*(\S+)", open(rs).read())
    if ref:
        blob = subprocess.run(
            ["git", "-C", X.REPO, "show",
             f"{ref.group(1)}:crates/kali_cli/tests/browser_{stem}.rs"],
            capture_output=True, text=True)
        if blob.returncode != 0:
            return None
        return blob.stdout.split("\n")
    return open(rs).read().split("\n")


def _elide(expr):
    """Elide only the argument lists that would be UNSAFE to carry verbatim.

    An argument list is kept when it is short and free of backslashes, because
    keeping it is what makes the needle set discriminate: `.arg("--max-threads")`
    resolves on {arg, --max-threads} and is pinned to one line, while the elided
    `.arg(...)` resolves on {arg} alone and would sit happily on any argv line in
    the file. A backslash is the one thing that cannot be carried -- `"0\\n"` is
    written `0\\n` in a `#` comment and `0\\\\n` in a multi-line TOML string, so a
    needle taken from it would be right on one surface and wrong on the other.
    """
    def sub(m):
        args = m.group(1)
        if not args.strip():
            return "()"
        if "\\" in args or len(args) > 45:
            return "(...)"
        return m.group(0)
    return re.sub(r"\(([^()]*)\)", sub, expr)


def _candidates(lines, first, last):
    """Quotable constructs inside the cited range, best first.

    Ordered by how much they discriminate: a `receiver.method(` yields two
    needles and is preferred over a bare index, which yields one.
    """
    _items = X._source_items("\n".join(lines))

    def harvest(source_lines):
        got = []
        for line in source_lines:
            for m in list(_EXPR.finditer(line)) + list(_OPEN_CALL.finditer(line)):
                e = _elide(m.group(0))
                if len(e) > _SNIPPET_BOUND or "\\" in e or "`" in e:
                    continue
                if e not in got:
                    got.append(e)
        return sorted(got, key=lambda e: (-len(X._needles(e, _items)), len(e)))

    cited = harvest(lines[first - 1:min(last, len(lines))])
    # FALLBACK: the enclosing statement. The gate resolves at enclosing-statement
    # granularity by design (ruling 11's corrected granularity clause), so a
    # construct anywhere in the same statement is a legitimate anchor for a
    # citation onto any of its lines -- a citation onto the `) {` of a
    # rustfmt-split signature is explicitly not drift. The cited lines are still
    # preferred, so this only fires where they carry nothing quotable at all.
    window = [l for l in X._statement(lines, first, min(last, len(lines))) if l]
    return cited + [e for e in harvest(window) if e not in cited]


def _resolves(snippet, lines, first, last):
    needles = X._needles(snippet, X._source_items("\n".join(lines)))
    if not needles:
        return False
    stmt = "\n".join(X._statement(lines, first, min(last, len(lines))))
    return all(tok in stmt for tok in needles)


def rework_text(stem, text):
    """The reword, as a pure function of (stem, case-file text).

    This is the whole of the reword, and it is called from ONE place at
    generation time -- `case_emit.write` -- so a generator cannot emit an
    ungated citation that a later post-pass has to repair. Everything below
    used to run only as that post-pass, which is exactly why the shipped tree
    stopped being reproducible: the generators emitted `(:N)` and the tree
    carried `` `snippet` (:N) ``.

    Returns `(text, done, failed)`. It is IDEMPOTENT: a citation that already
    carries a backticked construct is matched by `CITE`/`SUBMOD_CITE` and is
    therefore excluded from `sites`, so re-running over reworded text is a
    no-op. That is what lets the fold sit under every generator, including the
    three that were already fixed points.
    """
    lines = _pretrim_lines(stem, text)
    if lines is None:
        return text, [], [f"{stem}: no resolvable source"]
    covered = [(m.start(), m.end()) for m in
               list(X.SUBMOD_CITE.finditer(text)) + list(X.CITE.finditer(text))]
    sites = [m for m in X.WRITTEN_CITE.finditer(text)
             if not any(a <= m.start() < b for a, b in covered)]
    done, failed = [], []
    # RIGHT TO LEFT, so an earlier site's insertion cannot invalidate a later
    # site's offset. (The `.toml` is what moves here; the `.rs` line numbers the
    # citations point at are untouched by any edit in this file, which is why
    # this pass needs no fixed point -- unlike a `//!` header edit.)
    for m in reversed(sites):
        cite = m.group(0)
        num = re.match(r"(?:[A-Za-z0-9_]+\.rs)?:(\d+)(?:-(\d+))?", cite)
        first = int(num.group(1))
        last = int(num.group(2)) if num.group(2) else first
        if first > len(lines):
            failed.append(f"{stem}: {cite} is past end of source ({len(lines)} lines)"
                          " -- STALE, re-derive by hand")
            continue
        pick = next((c for c in _candidates(lines, first, last)
                     if _resolves(c, lines, first, last)), None)
        if pick is None:
            failed.append(f"{stem}: {cite} -- no construct in the cited range "
                          f"resolves; re-derive by hand "
                          f"(line {first}: {lines[first - 1].strip()[:70]!r})")
            continue
        text = text[:m.start()] + f"`{pick}` " + text[m.start():]
        done.append((cite, pick))
    return text, done, failed


def rework(stem, apply=False):
    """The on-disk driver. Kept so the standalone report/`--apply` run still
    works; the generation-time path goes through `rework_text` directly."""
    toml_path = os.path.join(CASES, f"{stem}.toml")
    text, done, failed = rework_text(stem, open(toml_path).read())
    if apply and done:
        with open(toml_path, "w") as f:
            f.write(text)
    return done, failed


def main(argv):
    apply = "--apply" in argv
    stems = [a for a in argv[1:] if not a.startswith("--")]
    if not stems:
        stems = sorted(f[:-5] for f in os.listdir(CASES) if f.endswith(".toml"))
    n_done = 0
    all_failed = []
    for stem in stems:
        done, failed = rework(stem, apply)
        all_failed += failed
        n_done += len(done)
        if done or failed:
            print(f"{stem}: {len(done)} reworded, {len(failed)} unresolvable")
    print(f"\nTOTAL: {n_done} citation(s) reworded, {len(all_failed)} unresolvable")
    for f in all_failed:
        print(f"  {f}")
    return 1 if all_failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
