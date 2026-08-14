#!/usr/bin/env python3
"""Snapshot / diff the assertion surface of every `cases/browser/*.toml`.

Batch 4's review called this the strongest single check in the project, so
batch 5 committed it rather than re-improvising it: a migration batch must move
this surface by *exactly* what it added, and a prose-only follow-up round must
not move it at all.

It deliberately reuses `scripts/audit-case-migration.py`'s OWN extraction
(`assertion_strings`, via `resolved_steps`) instead of re-deriving one. That is
the point: the audit gate's notion of "what this case file asserts" is the
thing whose drift matters, and a second implementation would diff its own
opinion instead. (The independent-second-extractor discipline of U12 is
served elsewhere -- `fidelity.py` and `check_extra_claims.py` -- and is not
what this tool is for.)

The unit counted is one (file, assertion-string) pair, as a multiset: the same
needle asserted by two different case files, or twice inside one file, counts
twice, because dropping one of them is a real lost claim.

Usage:
  assertion_diff.py snapshot OUT.json          # write the current surface
  assertion_diff.py diff BEFORE.json AFTER.json  # report the delta
"""

import collections
import importlib.util
import json
import os
import sys
import tomllib

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
CASES = os.path.join(REPO, "crates/kali_cli/tests/cases/browser")

_spec = importlib.util.spec_from_file_location(
    "audit_case_migration", os.path.join(REPO, "scripts/audit-case-migration.py")
)
_audit = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_audit)


def surface():
    """{case-file basename: sorted list of its assertion strings}."""
    out = {}
    for name in sorted(os.listdir(CASES)):
        if not name.endswith(".toml"):
            continue
        with open(os.path.join(CASES, name), "rb") as handle:
            doc = tomllib.load(handle)
        out[name] = sorted(_audit.assertion_strings(doc))
    return out


def _multiset(surf):
    counter = collections.Counter()
    for name, strings in surf.items():
        for s in strings:
            counter[(name, s)] += 1
    return counter


def main(argv):
    if len(argv) == 3 and argv[1] == "snapshot":
        surf = surface()
        with open(argv[2], "w") as handle:
            json.dump(surf, handle, indent=1, sort_keys=True)
        total = sum(len(v) for v in surf.values())
        print(f"snapshot: {len(surf)} case file(s), {total} assertion string(s) -> {argv[2]}")
        return 0
    if len(argv) == 4 and argv[1] == "diff":
        before = _multiset(json.load(open(argv[2])))
        after = _multiset(json.load(open(argv[3])))
        added = after - before
        removed = before - after
        files_before = {name for name, _ in before}
        files_after = {name for name, _ in after}
        print(f"case files: {len(files_before)} -> {len(files_after)} "
              f"(+{len(files_after - files_before)} / -{len(files_before - files_after)})")
        for name in sorted(files_after - files_before):
            n = sum(c for (f, _), c in after.items() if f == name)
            print(f"  NEW FILE   {name}: {n} assertion string(s)")
        for name in sorted(files_before - files_after):
            n = sum(c for (f, _), c in before.items() if f == name)
            print(f"  GONE FILE  {name}: {n} assertion string(s)")
        pre_existing_added = {k: v for k, v in added.items() if k[0] in files_before}
        pre_existing_removed = {k: v for k, v in removed.items() if k[0] in files_after}
        print(f"total assertion strings: {sum(before.values())} -> {sum(after.values())} "
              f"(added {sum(added.values())}, removed {sum(removed.values())})")
        print(f"CHANGES TO PRE-EXISTING FILES: "
              f"{sum(pre_existing_added.values())} added, "
              f"{sum(pre_existing_removed.values())} removed")
        for (name, s), c in sorted(pre_existing_added.items()):
            print(f"  + {name}: {s!r} x{c}")
        for (name, s), c in sorted(pre_existing_removed.items()):
            print(f"  - {name}: {s!r} x{c}")
        return 0
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv))
