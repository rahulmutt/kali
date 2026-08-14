#!/usr/bin/env python3
"""What a migration FAMILY is, in one place, for every gate that needs one.

A family is a subdirectory of `crates/kali_cli/tests/cases`. Its case files
are `cases/<family>/<stem>.toml` and their sources are
`crates/kali_cli/tests/<prefix><stem>.rs`.

    python3 families.py --list              # every family, with its prefix
    python3 families.py --prefix browser    # -> browser_
    python3 families.py --prefix misc       # -> (empty)
    python3 families.py --selftest          # the gate

WHY A MODULE AND NOT A CONSTANT PER TOOL. Until now `browser_` was spelled
into `verify_pair.sh`, `citation_sweep.sh`, `batch5_crosscheck.py` and
`source_ref_rehearsal.py` separately -- four copies of one fact, and the
Task 19 pilot's verdict was that none of those tools can run on a non-browser
pair at all. Batch 2 migrates ~47 targets across several families.

THE PREFIX IS DERIVED, NOT TABULATED (ruling 18 #1). A table mapping
`misc -> ""` and everything else to `<family>_` is a mark: it says nothing
about the tree and goes stale silently the day a family stops matching it.
Instead each family's prefix is read off its OWN shipped case files -- for
every `cases/<family>/<stem>.toml`, the source named in its `Migrated from
tests/...` header is either `<family>_<stem>.rs` (prefix `<family>_`) or
`<stem>.rs` (prefix empty). A U2 split, whose case stem deliberately differs
from its source stem, votes for neither and is skipped.

A NON-UNANIMOUS FAMILY IS AN ERROR, not a majority vote (ruling 18 #3): if
two case files in one directory disagree about their family's prefix, no
prefix is right and a gate that picks one resolves half the family against
the wrong filename. `--prefix` then fails and the caller must say which it
means. Measured across the whole corpus at the time of writing: all ten
families are unanimous, and `--selftest` re-derives that every run rather
than trusting this sentence.

A family with no case files yet cannot be derived from -- the first
migration into a new family passes its prefix explicitly.
"""

from __future__ import annotations

import argparse
import glob
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
TESTS = os.path.join(REPO, "crates", "kali_cli", "tests")
CASES_ROOT = os.path.join(TESTS, "cases")

# THE ONE DEFINITION of "this header names a source", and it is a definition
# rather than a claim about other files. An earlier version of this comment said
# "the same regex `citation_tiers` and `batch5_crosscheck` use" -- which was a
# MARK, and false: all three spellings differed. In a module whose whole thesis
# is that marks go stale, that is the wrong way round. So `batch5_crosscheck`
# and `source_ref_rehearsal` now IMPORT this pattern instead of restating it,
# and the sentence is true because nothing else can spell it.
#
# `MIGRATED_FROM_PATTERN` is exported separately because
# `source_ref_rehearsal` needs to anchor it to a `#` comment line.
MIGRATED_FROM_PATTERN = r"Migrated from tests/([A-Za-z0-9_./]+\.rs)"
MIGRATED_FROM = re.compile(MIGRATED_FROM_PATTERN)


class FamilyError(Exception):
    pass


def families() -> list[str]:
    """Every family directory under `cases/`, sorted."""
    if not os.path.isdir(CASES_ROOT):
        raise FamilyError(f"{CASES_ROOT} does not exist")
    return sorted(name for name in os.listdir(CASES_ROOT)
                  if os.path.isdir(os.path.join(CASES_ROOT, name)))


def cases_dir(family: str) -> str:
    path = os.path.join(CASES_ROOT, family)
    if not os.path.isdir(path):
        raise FamilyError(
            f"no such family {family!r}: {path} is not a directory. "
            f"Known families: {', '.join(families())}")
    return path


def case_files(family: str) -> list[str]:
    return sorted(glob.glob(os.path.join(cases_dir(family), "*.toml")))


def prefix(family: str) -> str:
    """The source-filename prefix for `family`, derived from its case files.

    Raises `FamilyError` when the family has no case file that votes, or when
    its case files disagree."""
    votes: dict[str, list[str]] = {}
    for toml in case_files(family):
        stem = os.path.basename(toml)[:-len(".toml")]
        named = MIGRATED_FROM.search(open(toml).read())
        if not named:
            continue
        name = named.group(1)
        if name == f"{family}_{stem}.rs":
            votes.setdefault(f"{family}_", []).append(stem)
        elif name == f"{stem}.rs":
            votes.setdefault("", []).append(stem)
        # anything else is a U2 split (case stem != source stem) and abstains
    if not votes:
        raise FamilyError(
            f"cannot derive a source prefix for family {family!r}: none of its "
            f"{len(case_files(family))} case file(s) names a source whose stem "
            f"matches its own. Pass the prefix explicitly.")
    if len(votes) > 1:
        detail = "; ".join(f"{p!r} from {sorted(v)[0]} (+{len(v) - 1} more)"
                           for p, v in sorted(votes.items()))
        raise FamilyError(
            f"family {family!r} does not agree with itself about its source "
            f"prefix: {detail}. No prefix is right for the whole directory; "
            f"pass one explicitly and split the family if that is wrong.")
    return next(iter(votes))


def source_name(family: str, stem: str, family_prefix: str | None = None) -> str:
    """`<prefix><stem>.rs` -- the source filename, relative to `tests/`."""
    return f"{prefix(family) if family_prefix is None else family_prefix}{stem}.rs"


def marker(family: str, family_prefix: str | None = None) -> str:
    """The `Migrated from` marker prefix a case file in this family carries."""
    p = prefix(family) if family_prefix is None else family_prefix
    return f"Migrated from tests/{p}"


def family_of(case_path: str) -> str:
    """The family a case file belongs to, from its own path."""
    return os.path.basename(os.path.dirname(os.path.abspath(case_path)))


def _selftest() -> int:
    global CASES_ROOT
    problems: list[str] = []
    found = families()
    print(f"  {len(found)} family/families under {os.path.relpath(CASES_ROOT, REPO)}")
    for family in found:
        try:
            p = prefix(family)
        except FamilyError as error:
            problems.append(str(error))
            continue
        n = len(case_files(family))
        print(f"    {family:<10} prefix={p!r:<14} {n} case file(s)")

    # A KNOWN POSITIVE and a KNOWN NEGATIVE. An instrument validated in one
    # direction only passes trivially by answering that way always.
    #
    # THE ABSENCE OF EITHER FAMILY IS A FAILURE, not a reason to skip the
    # assertion (review, minor). Guarding these with `if "browser" in found`
    # meant the two checks that give this selftest its content would evaporate
    # silently the day someone renamed or removed a directory -- leaving a green
    # selftest that asserts nothing about any prefix at all.
    if "browser" not in found:
        problems.append("no `browser` family -- the known-positive prefix check "
                        "cannot run, so this selftest would pass vacuously")
    elif prefix("browser") != "browser_":
        problems.append("browser's prefix is not `browser_`")
    if "misc" not in found:
        problems.append("no `misc` family -- the empty-prefix check cannot run, "
                        "and it is the one that proves the prefix is DERIVED "
                        "rather than assumed to be `<family>_`")
    elif prefix("misc") != "":
        problems.append("misc's prefix is not empty -- misc/ sources carry no "
                        "family prefix, which is the whole reason the prefix is "
                        "derived rather than assumed to be `<family>_`")

    # THE NEGATIVE: a family that disagrees with itself must RAISE, not vote.
    import shutil
    import tempfile
    real = CASES_ROOT
    scratch = None
    try:
        scratch = tempfile.mkdtemp(prefix="families_selftest_")
        os.makedirs(os.path.join(scratch, "mixed"))
        with open(os.path.join(scratch, "mixed", "a.toml"), "w") as handle:
            handle.write("# Migrated from tests/mixed_a.rs\n")
        with open(os.path.join(scratch, "mixed", "b.toml"), "w") as handle:
            handle.write("# Migrated from tests/b.rs\n")
        CASES_ROOT = scratch
        try:
            got = prefix("mixed")
            problems.append(f"a self-contradicting family returned {got!r} "
                            f"instead of raising")
            print("  poison -- self-contradicting family: NOT caught")
        except FamilyError:
            print("  poison -- self-contradicting family: raises")
        # And a family nothing votes for.
        os.makedirs(os.path.join(scratch, "empty"))
        try:
            prefix("empty")
            problems.append("an underivable family returned a prefix instead "
                            "of raising")
            print("  poison -- family with no votes: NOT caught")
        except FamilyError:
            print("  poison -- family with no votes: raises")
    finally:
        CASES_ROOT = real
        if scratch:
            shutil.rmtree(scratch, ignore_errors=True)

    if problems:
        print("\nFAMILIES SELFTEST FAILED")
        for problem in problems:
            print(f"  {problem}")
        return 1
    print("\nFAMILIES SELFTEST OK — every family derives one unanimous source "
          "prefix, browser's is `browser_` and misc's is empty, and a family "
          "that disagrees with itself raises rather than voting")
    return 0


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--list", action="store_true")
    group.add_argument("--prefix", metavar="FAMILY")
    group.add_argument("--cases-dir", metavar="FAMILY")
    group.add_argument("--selftest", action="store_true")
    args = parser.parse_args(argv)
    try:
        if args.selftest:
            return _selftest()
        if args.list:
            for family in families():
                try:
                    print(f"{family}\t{prefix(family)}")
                except FamilyError as error:
                    print(f"{family}\t<underivable: {error}>")
            return 0
        if args.cases_dir:
            print(os.path.relpath(cases_dir(args.cases_dir), REPO))
            return 0
        print(prefix(args.prefix))
        return 0
    except FamilyError as error:
        print(f"error: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
