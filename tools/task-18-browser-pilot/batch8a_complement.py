#!/usr/bin/env python3
"""Ruling 12's third left-hand side, for a `#[path]` SUBMODULE-CARRIER trim.

`migrated_complement.py` builds the migrated half of a U4 trim by subtracting
the retained file's top-level `fn` items from the pre-trim blob's. That is the
whole answer for a flat `.rs`. It is only HALF the answer for a submodule
carrier, and the missing half is the important one: the carrier holds the
helpers, but every `#[test]` fn lives in the sibling directory, so the
item-subtraction alone yields a source with ZERO tests and
`audit-case-migration.py` exits 2 on "0 `#[test]` fns found" rather than
auditing anything.

`browser_reflect_own_keys` is the first trim of that shape in this family. Its
retained half is the `test.rs` submodule (16 fns) plus the carrier items that
submodule reaches; its migrated half is the `run.rs`/`build.rs`/`check.rs`
submodules (28 fns) plus the carrier items only they reached.

So the complement is built in two pieces and joined the way
`audit-case-migration.py`'s own `main` joins a carrier with its submodules
(newline-separated, carrier first), which is what makes the audit see exactly
the corpus it would have seen pre-trim minus the retained half:

    complement = migrated_complement(carrier_pretrim, carrier_retained)
               + each migrated submodule's text, read from the pre-trim ref

BOTH SIDES COME FROM THINGS THAT ACTUALLY EXIST -- the pre-trim blob named by
the retained file's own `PRE-TRIM REF:` line, and the shipped retained files --
never from a hand-maintained list of what was migrated, which could drift from
either. The ref is read from the header rather than hardcoded here, for the
reason `case_emit.source_text` records: a ref carried anywhere but the header is
the moving figure ruling 11 forbids.

It exits non-zero if the retained carrier is not a subset of the pre-trim
carrier by item name, or if a submodule it expects to be migrated is still on
disk (which would mean the trim did not happen the way this script assumes).

Usage:
  batch8a_complement.py                 # build it and RUN the audit against it
  batch8a_complement.py --print         # write the complement to stdout only
"""

import os
import re
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
TESTS = os.path.join(REPO, "crates/kali_cli/tests")

from migrated_complement import split_items  # noqa: E402

STEM = "reflect_own_keys"
CARRIER = f"crates/kali_cli/tests/browser_{STEM}.rs"
RETAINED_SUBS = ["test.rs"]
MIGRATED_SUBS = ["run.rs", "build.rs", "check.rs"]
CASES = ["cases/browser/reflect_own_keys_explicit_api.toml",
         "cases/browser/reflect_own_keys_inherited_manifest.toml"]


def pretrim_ref():
    text = open(os.path.join(REPO, CARRIER)).read()
    m = re.search(r"PRE-TRIM REF:\s*([0-9a-f]{40})\b", text)
    if not m:
        raise SystemExit(
            f"{CARRIER} declares no full-sha PRE-TRIM REF in its header; this script "
            "will not guess one.")
    return m.group(1)


def show(ref, path):
    p = subprocess.run(["git", "show", f"{ref}:{path}"], cwd=REPO,
                       capture_output=True, text=True)
    if p.returncode != 0:
        raise SystemExit(f"cannot read {path} at {ref}: {p.stderr.strip()}")
    return p.stdout


def build():
    ref = pretrim_ref()
    retained_carrier = open(os.path.join(REPO, CARRIER)).read()
    pre_carrier = show(ref, CARRIER)

    for sub in MIGRATED_SUBS:
        on_disk = os.path.join(TESTS, f"browser_{STEM}", sub)
        if os.path.exists(on_disk):
            raise SystemExit(
                f"{on_disk} still exists -- this script assumes it was migrated and "
                "removed by the trim. Either the trim did not happen or the retained "
                "set is not what this script expects.")

    # Piece 1: the carrier items the trim removed.
    pre_pre, pre_items = split_items(pre_carrier)
    _ret_pre, ret_items = split_items(retained_carrier)
    unknown = [n for n in ret_items if n not in pre_items]
    if unknown:
        raise SystemExit(
            f"retained carrier has item(s) absent from the pre-trim carrier: {unknown} "
            "-- the two are not a trim of one another")
    removed = [n for n in pre_items if n not in ret_items]
    if not removed:
        raise SystemExit("no carrier item was removed -- not a carrier trim")

    # The preamble carries the `use` lines the removed helpers need; the `#[path]`
    # mod block is deliberately NOT carried, because the submodule TEXT is
    # appended directly below instead (resolving a `#[path]` from a temp file
    # would look outside the tests directory and find nothing).
    # `split_items` folds the trailing `#[path] mod` block into the LAST item's
    # text, and that item is one of the removed ones -- so the declarations ride
    # along unless they are stripped. They must be: the submodule TEXT is
    # inlined below instead, and a `#[path]` resolved from a temp directory
    # finds nothing and makes the audit exit 2 on a file that is otherwise
    # correct. (Observed, not anticipated.)
    removed_text = "\n".join(pre_items[n] for n in removed)
    removed_text = re.sub(r'#\[path = "[^"]+"\]\s*\n\s*mod \w+;\n?', "", removed_text)
    if "#[path" in removed_text or re.search(r"^\s*mod \w+;", removed_text, re.M):
        raise SystemExit("a `mod` declaration survived the strip; the complement would "
                         "resolve it against a temp directory and exit 2")
    pieces = [pre_pre.rstrip("\n"), removed_text]

    # Piece 2: every migrated submodule, at the pre-trim ref.
    for sub in MIGRATED_SUBS:
        text = show(ref, f"crates/kali_cli/tests/browser_{STEM}/{sub}")
        # `use super::*;` is meaningless once the text is inlined into one file.
        pieces.append("\n".join(l for l in text.split("\n")
                                if l.strip() != "use super::*;"))

    complement = "\n".join(pieces).rstrip("\n") + "\n"
    n_tests = sum(1 for l in complement.split("\n") if l.strip() == "#[test]")
    if n_tests != 28:
        raise SystemExit(
            f"complement holds {n_tests} `#[test]` fn(s), expected the 28 that were "
            "migrated -- the split this script assumes is not the split that happened")
    return ref, complement, removed, n_tests


def main(argv):
    ref, complement, removed, n_tests = build()
    if "--print" in argv:
        sys.stdout.write(complement)
        return 0
    print(f"PRE-TRIM REF {ref}")
    print(f"carrier items removed by the trim: {', '.join(removed)}")
    print(f"migrated submodules inlined: {', '.join(MIGRATED_SUBS)}")
    print(f"complement holds {n_tests} `#[test]` fn(s)\n")
    with tempfile.TemporaryDirectory() as d:
        path = os.path.join(d, f"browser_{STEM}.rs")
        with open(path, "w") as f:
            f.write(complement)
        cmd = ([sys.executable, os.path.join(REPO, "scripts/audit-case-migration.py"),
                path] + [os.path.join(TESTS, c) for c in CASES])
        p = subprocess.run(cmd, cwd=REPO, capture_output=True, text=True)
        sys.stdout.write(p.stdout)
        sys.stderr.write(p.stderr)
        if p.returncode != 0:
            print("\nAUDIT AGAINST THE MIGRATED COMPLEMENT FAILED", file=sys.stderr)
        return p.returncode


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
