#!/usr/bin/env python3
"""Reconstruct the MIGRATED HALF of a U4 trim-and-keep source, for auditing.

WHY THIS EXISTS. Ruling 9 says a U4 retention pair's gates are run against the
PRE-TRIM blob, because the case file was migrated from the file as it stood
before the trim. That is right for `comment_coverage.py` and for citations. It
is NOT right for `audit-case-migration.py` or `check_fixtures.py` whenever the
RETAINED tests carry literal claims of their own:

  * against the POST-trim `.rs`, the audit compares the case file's claims with
    a source stripped of the half that makes them -> red;
  * against the PRE-trim blob, it compares BOTH halves' claims with a case file
    that carries only the migrated half's -> also red.

THE CONDITION IN THAT SENTENCE IS LOAD-BEARING, AND TWO EARLIER VERSIONS OF THIS
PARAGRAPH DROPPED IT IN OPPOSITE DIRECTIONS. The first claimed batch 5's trims
were all green "because their retained tests' needles were loop variables" and
that batch 6A's trim was "the first" where they are not -- too narrow. Its
replacement then said "every already-adjudicated trim in this family is red on
its own pre-trim audit" -- too broad. Neither was measured; both were one
command per file away from being measured.

Measured, over every stem in the family carrying a `PRE-TRIM REF:` and a case
file -- TEN of them -- each against the ref in its OWN header, `audit` and
`check_fixtures` both run against the pre-trim blob:

  trim (ref from its own header)                     pre-trim  needs this script
  browser_array_iteration_spread          f0bfb76d79^  green     no
  browser_math_floor_trunc_ceil_aliases   b44fd6acf9^  green     no
  browser_math_floor_trunc_ceil_bundle    b44fd6acf9^  green     no
  browser_math_pow_bracketed_frozen_
      wrapper_harness                     f712bdbf4b   green     no
  browser_math_pow_bracketed_frozen_
      wrapper                             f712bdbf4b   green     no
  browser_math_abs_sign_frozen_aliases    1db95b469f^  audit red      yes
  browser_math_atan2_global_this_root     1db95b469f^  audit red      yes
  browser_math_max_min_frozen_aliases     f712bdbf4b   audit red      yes
  browser_math_pow_exponent_one           d7fc768c1f^  audit + fixtures red   yes
  browser_math_unsupported_member_calls_
      harness_jsx_tsx (batch 6A)          fe6a403411   audit + fixtures red   yes

So: FIVE of the ten need a third left-hand side and five do not, and the
discriminator is the condition above -- whether the RETAINED half carries
literal claims -- not the fact of being a trim. Of the five that do, only two
are red on pre-trim `check_fixtures` as well; the other three are red on the
audit alone. All five go green against the complement this script builds.

SCOPE FOR BATCH 7, STATED EXACTLY BECAUSE A WRONG SCOPE IN A HANDOFF SENTENCE IS
HOW THIS WENT WRONG TWICE. The retroactive header sweep is FOUR files, not ten
and not five: `browser_math_max_min_frozen_aliases.rs`,
`browser_math_abs_sign_frozen_aliases.rs`,
`browser_math_atan2_global_this_root.rs` and `browser_math_pow_exponent_one.rs`.
Those four, and only those, carry a sentence saying their audit red is "the
escalation itself, not a trim artifact", which is what needs correcting. The
five green trims say no such thing and need no edit. The retentions themselves
stand unchanged: all four are adjudicated on the FIXTURE SELF-INSPECTION ground
and all four are in `find_fixture_self_inspection.py`'s `KNOWN` list. The audit
red was never their escalation ground.

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
