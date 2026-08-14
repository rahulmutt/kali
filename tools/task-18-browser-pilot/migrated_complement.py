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

A `#[path]` SUBMODULE CARRIER NEEDS MORE THAN ITEM SUBTRACTION, and `--carrier`
is that mode. The carrier holds the helpers but every `#[test]` fn lives in the
sibling directory, so subtracting carrier items alone yields a source with ZERO
tests and `audit-case-migration.py` exits 2 on "0 `#[test]` fns found" rather
than auditing anything. `--carrier` appends each MIGRATED submodule's pre-trim
text below the removed carrier items, joined the way
`audit-case-migration.py`'s own `main` joins a carrier with its submodules, so
the audit sees exactly the corpus it would have seen pre-trim minus the retained
half. Which submodules are migrated is DERIVED -- the pre-trim carrier's
`#[path]` set minus whatever still resolves from the retained one -- not listed,
so it cannot drift from the trim that actually happened.

Both sides come from things that exist: the blob at the ref the RETAINED file's
own header declares, and the shipped retained files. The ref is read from the
header rather than passed, for the reason `case_emit.source_text` records -- a
ref carried anywhere but the header is the moving figure ruling 11 forbids.

Usage:
  migrated_complement.py PRETRIM.rs RETAINED.rs > MIGRATED_PART.rs
  migrated_complement.py --carrier RETAINED.rs > MIGRATED_PART.rs
  migrated_complement.py --carrier RETAINED.rs --audit TARGET.toml [TARGET.toml ...]
"""

import os
import re
import subprocess
import sys
import tempfile

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


def _git_show(repo, ref, path):
    p = subprocess.run(["git", "show", f"{ref}:{path}"], cwd=repo,
                       capture_output=True, text=True)
    if p.returncode != 0:
        raise SystemExit(f"cannot read {path} at {ref}: {p.stderr.strip()}")
    return p.stdout


def carrier_complement(retained_path):
    """(complement text, removed item names, migrated submodule names, ref)."""
    here = os.path.dirname(os.path.abspath(__file__))
    repo = os.path.abspath(os.path.join(here, "..", ".."))
    sys.path.insert(0, here)
    from submodules import submodule_paths

    retained = open(retained_path).read()
    m = re.search(r"PRE-TRIM REF:\s*([0-9a-f]{40})\b", retained)
    if not m:
        raise SystemExit(
            f"{retained_path} declares no full-sha `PRE-TRIM REF:` in its header; "
            "this tool will not guess one.")
    ref = m.group(1)
    rel = os.path.relpath(os.path.abspath(retained_path), repo)
    pretrim = _git_show(repo, ref, rel)

    pre_preamble, pre_items = split_items(pretrim)
    _rp, ret_items = split_items(retained)
    unknown = [n for n in ret_items if n not in pre_items]
    if unknown:
        raise SystemExit(f"retained carrier has item(s) absent from the pre-trim "
                         f"carrier: {unknown} -- not a trim of one another")
    removed = [n for n in pre_items if n not in ret_items]

    # DERIVED: the submodules the pre-trim carrier declared, minus the ones the
    # retained carrier still resolves. No hardcoded list to drift.
    pre_subs = re.findall(r'#\[path = "([^"]+)"\]', pretrim)
    still = {p.name for p in submodule_paths(retained_path)}
    migrated = [s for s in pre_subs if os.path.basename(s) not in still]
    if not migrated and not removed:
        raise SystemExit("nothing was removed -- a whole-file retention, not a trim")

    # The `#[path] mod` block sits after the last fn, so `split_items` folds it
    # into that item's text and it rides along unless stripped. It must be: the
    # submodule TEXT is inlined below instead, and a `#[path]` resolved from a
    # temp directory finds nothing, which makes the audit exit 2 on a file that
    # is otherwise correct.
    removed_text = "\n".join(pre_items[n] for n in removed)
    removed_text = re.sub(r'#\[path = "[^"]+"\]\s*\n\s*mod \w+;\n?', "", removed_text)
    if "#[path" in removed_text or re.search(r"^\s*mod \w+;", removed_text, re.M):
        raise SystemExit("a `mod` declaration survived the strip")

    pieces = [pre_preamble.rstrip("\n"), removed_text]
    for sub in migrated:
        text = _git_show(repo, ref, f"crates/kali_cli/tests/{sub}")
        pieces.append("\n".join(l for l in text.split("\n")
                                if l.strip() != "use super::*;"))
    return "\n".join(pieces).rstrip("\n") + "\n", removed, migrated, ref


def _carrier_main(argv):
    retained = argv[0]
    text, removed, migrated, ref = carrier_complement(retained)
    n_tests = sum(1 for l in text.split("\n") if l.strip() == "#[test]")
    if "--audit" not in argv:
        sys.stdout.write(text)
        print(f"carrier complement at {ref}: removed item(s) {', '.join(removed)}; "
              f"migrated submodule(s) {', '.join(migrated)}; {n_tests} `#[test]` fn(s)",
              file=sys.stderr)
        return 0
    tomls = argv[argv.index("--audit") + 1:]
    if not tomls:
        raise SystemExit("--audit needs at least one TARGET.toml")
    here = os.path.dirname(os.path.abspath(__file__))
    repo = os.path.abspath(os.path.join(here, "..", ".."))
    print(f"PRE-TRIM REF {ref}")
    print(f"carrier items removed by the trim: {', '.join(removed)}")
    print(f"migrated submodules inlined: {', '.join(migrated)}")
    print(f"complement holds {n_tests} `#[test]` fn(s)\n")
    with tempfile.TemporaryDirectory() as d:
        path = os.path.join(d, os.path.basename(retained))
        with open(path, "w") as f:
            f.write(text)
        p = subprocess.run(
            [sys.executable, os.path.join(repo, "scripts/audit-case-migration.py"),
             path] + [os.path.abspath(t) for t in tomls],
            cwd=repo, capture_output=True, text=True)
        sys.stdout.write(p.stdout)
        sys.stderr.write(p.stderr)
        if p.returncode != 0:
            print("\nAUDIT AGAINST THE MIGRATED COMPLEMENT FAILED", file=sys.stderr)
        return p.returncode


def main(argv):
    if argv and argv[0] == "--carrier":
        if len(argv) < 2:
            raise SystemExit(__doc__)
        return _carrier_main(argv[1:])
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
