#!/usr/bin/env python3
"""Reconstruct the MIGRATED HALF of a U4 trim-and-keep source, for auditing.

WHY THIS EXISTS. Ruling 9 says a U4 retention pair's gates are run against the
PRE-TRIM blob, because the case file was migrated from the file as it stood
before the trim. That is right for `comment_coverage.py` and for citations, and
it was sufficient for every retention shipped through batch 5 -- but only by
accident of those files' contents. It is NOT sufficient for
`audit-case-migration.py` or `check_fixtures.py` whenever the RETAINED tests
carry literal claims of their own:

  * against the POST-trim `.rs`, the audit compares the retained half's claims
    against a case file that by construction does not carry them -> red;
  * against the PRE-trim blob, it compares BOTH halves' claims against a case
    file that carries only the migrated half's -> also red.

Batch 5's three trims were green on both sides because their retained tests'
needles were loop variables, so the retained half contributed no literal. Batch
6A's `browser_math_unsupported_member_calls_harness_jsx_tsx.rs` is the first
where it does: the three retained tests assert `E5506`, `Math.sqrt`,
`Math.atan2`, `unsupported math` and the JSON key `code`, none of which any
migrated case may claim.

The right left-hand side is neither blob but their DIFFERENCE: the pre-trim
source minus the retained half, i.e. exactly the part that was migrated. This
builds it, mechanically, from the two things that actually exist -- the pre-trim
blob and the shipped retained `.rs` -- rather than from a hand-maintained list
that could drift from either.

METHOD. Split each file into top-level items (an item is a `fn`, with any
attributes and `///` docs immediately above it; everything before the first item
is the preamble). The complement is the preamble plus every pre-trim item whose
`fn` name does not appear in the retained file. Item-name based, so it is
insensitive to reordering and to the `//!` header the retention added.

It exits non-zero if the retained file is not a subset of the pre-trim one by
name, which is the way this could silently produce a wrong answer.

Usage:
  migrated_complement.py PRETRIM.rs RETAINED.rs > MIGRATED_PART.rs
"""

import re
import sys

ITEM = re.compile(r"^(?:pub\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)", re.M)


def split_items(text):
    """(preamble, {fn_name: source_text}) in source order."""
    lines = text.split("\n")
    starts = []
    for i, line in enumerate(lines):
        m = ITEM.match(line)
        if m:
            # Walk back over the attributes and `///` docs that belong to it.
            j = i
            while j > 0:
                prev = lines[j - 1].strip()
                if prev.startswith("#[") or prev.startswith("///") or (
                        prev.startswith(")") and prev.endswith("{")):
                    j -= 1
                else:
                    break
            starts.append((j, i, m.group(1)))
    if not starts:
        return text, {}
    preamble = "\n".join(lines[:starts[0][0]])
    items = {}
    for k, (start, _line, name) in enumerate(starts):
        end = starts[k + 1][0] if k + 1 < len(starts) else len(lines)
        items[name] = "\n".join(lines[start:end])
    return preamble, items


def main(argv):
    if len(argv) != 2:
        raise SystemExit(__doc__)
    pretrim = open(argv[0]).read()
    retained = open(argv[1]).read()
    pre_preamble, pre_items = split_items(pretrim)
    _ret_preamble, ret_items = split_items(retained)
    if not pre_items:
        print("no top-level fn found in the pre-trim source", file=sys.stderr)
        return 2
    unknown = [n for n in ret_items if n not in pre_items]
    if unknown:
        print(f"retained file has item(s) absent from the pre-trim source: {unknown} -- "
              "the two files are not a trim of one another", file=sys.stderr)
        return 2
    kept = [name for name in pre_items if name not in ret_items]
    if not kept:
        print("every item was retained -- this is a whole-file retention, not a trim",
              file=sys.stderr)
        return 2
    out = [pre_preamble]
    out += [pre_items[name] for name in kept]
    sys.stdout.write("\n".join(out).rstrip() + "\n")
    print(f"migrated complement: {len(kept)} of {len(pre_items)} item(s) "
          f"({', '.join(kept)})", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
