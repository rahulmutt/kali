#!/usr/bin/env python3
"""Run `scripts/audit-case-migration.py` over EVERY shipped case file, and
diff two revisions of that script's verdicts over the same corpus.

WHY THIS EXISTS. `audit-case-migration.py` is the gate rule 3 calls absolute,
and every migration in this project has passed through it. A change to it is
therefore a change to the meaning of every shipped pair's `AUDIT OK` -- but
until now there was no way to ask "did any of those verdicts move?", because
the answer needs a source for each of the 247 case files and most of those
sources have been deleted. This resolves them and asks.

    python3 tools/migration/audit_corpus_sweep.py            # the sweep
    python3 tools/migration/audit_corpus_sweep.py --compare REF
    python3 tools/migration/audit_corpus_sweep.py --selftest # the gate

`--compare REF` runs REF's OWN copy of the audit script over today's corpus
as well, and exits non-zero unless every pair's return code AND every byte of
its output agree. That is ruling 15's answer 2 -- both sides pinned: the
left-hand side is a git ref, the right-hand side is the working tree, and the
report that quotes a result quotes both shas.

HOW A SOURCE IS RESOLVED, in order, per case file:

  1. `crates/kali_cli/tests/<name>.rs` in the working tree, where `<name>` is
     read from the case file's own `Migrated from tests/<name>` header;
  2. else the blob at the file's declared `SOURCE REF:` -- the Task 18
     mechanism, present on 155 of the 247 case files at the time of writing;
  3. else the parent of the commit that deleted that path, found with
     `git log --diff-filter=D`. This covers the Task 16/17-era families
     (array, math, object, soundness, string, switch), which deleted their
     sources before the `SOURCE REF:` convention existed.

A source resolved out of history is materialised as a whole
`crates/kali_cli/tests` tree, not a single blob, because a U10 `#[path]`
carrier's submodules have to resolve beside it -- the same reason
`citation_tiers.py --resolve-source` reproduces the whole tree.

**A case file whose source cannot be resolved is an ERROR, never a skip**
(ruling 18 #3). A sweep that silently drops what it cannot resolve reports a
smaller, greener corpus than it audited, and nothing distinguishes that from
having audited everything.

FAMILY-AGNOSTIC BY CONSTRUCTION: the corpus is `cases/*/*.toml` and the
source name comes from each file's own header, so a new family needs no
change here. Case files migrated from the SAME source are audited together,
which is what a U2 split requires (auditing one half alone reports the other
half's claims as dropped).

NOT IN `test-gate.sh --gates-only`, deliberately, for the same reason
`classify_drift.py`'s census is not: it shells out once per pair and
materialises four historical trees. Its `--selftest` is cheap and is the
gating arm; the differential is a clean-tree invocation quoted with both
shas.
"""

from __future__ import annotations

import argparse
import collections
import glob
import os
import re
import shutil
import subprocess
import sys
import tempfile

REPO = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
TESTS = os.path.join(REPO, "crates", "kali_cli", "tests")
CASES = os.path.join(TESTS, "cases")
AUDIT_REL = "scripts/audit-case-migration.py"

MIGRATED_FROM = re.compile(r"Migrated from tests/([A-Za-z0-9_./]+\.rs)")
SOURCE_REF = re.compile(r"SOURCE REF:\s*([0-9a-f]{40})")

# Every audit line that names a filesystem path names the SOURCE, and a
# source resolved out of history lives in a fresh temporary directory whose
# name differs between two runs. Normalising it is the difference between
# "216 of 236 pairs differ" and the real answer; without it the comparison
# reports noise as drift and a reader cannot tell which it is looking at.
_TMP_SOURCE = re.compile(r"\S*/crates/kali_cli/tests/")


def _normalise(text: str) -> str:
    return _TMP_SOURCE.sub("<TESTS>/", text)


def _run(*argv: str, **kw) -> subprocess.CompletedProcess:
    return subprocess.run(argv, capture_output=True, text=True, **kw)


def _deletion_parent(name: str) -> str | None:
    """The commit just before the one that deleted `tests/<name>`."""
    out = _run("git", "-C", REPO, "log", "--diff-filter=D", "--format=%H", "-1",
               "--", f"crates/kali_cli/tests/{name}").stdout.strip()
    return f"{out}^" if out else None


def pairs() -> list[tuple[str, str | None, list[str]]]:
    """`(source name, ref or None, [case file, ...])`, one entry per source,
    sorted. `ref is None` means the source is in the working tree."""
    groups: collections.defaultdict[tuple[str, str | None], list[str]] = \
        collections.defaultdict(list)
    unresolved: list[str] = []
    for toml in sorted(glob.glob(os.path.join(CASES, "*", "*.toml"))):
        text = open(toml).read()
        named = MIGRATED_FROM.search(text)
        if not named:
            unresolved.append(f"{os.path.relpath(toml, REPO)}: no `Migrated from tests/...`")
            continue
        name = named.group(1)
        ref: str | None = None
        if not os.path.exists(os.path.join(TESTS, name)):
            declared = SOURCE_REF.search(text)
            ref = declared.group(1) if declared else _deletion_parent(name)
            if ref is None:
                unresolved.append(
                    f"{os.path.relpath(toml, REPO)}: source tests/{name} is not in "
                    f"the tree, declares no `SOURCE REF:`, and no commit in this "
                    f"history deletes it")
                continue
        groups[(name, ref)].append(toml)
    if unresolved:
        sys.exit("SWEEP FAILED — cannot resolve a source for:\n  "
                 + "\n  ".join(unresolved))
    return [(name, ref, tomls) for (name, ref), tomls in sorted(groups.items())]


def _tree_at(ref: str, cache: dict[str, str], workdir: str) -> str:
    """`crates/kali_cli/tests` as of `ref`, materialised once per ref."""
    if ref not in cache:
        dest = tempfile.mkdtemp(prefix="tests_", dir=workdir)
        archive = subprocess.run(
            ["git", "-C", REPO, "archive", ref, "--", "crates/kali_cli/tests"],
            capture_output=True)
        if archive.returncode:
            sys.exit(f"cannot archive {ref}: {archive.stderr.decode().strip()}")
        subprocess.run(["tar", "-x", "-C", dest], input=archive.stdout, check=True)
        cache[ref] = os.path.join(dest, "crates", "kali_cli", "tests")
    return cache[ref]


def sweep(audit: str, workdir: str, cache: dict[str, str]) -> dict[str, tuple[int, str]]:
    """`{pair key: (rc, normalised stdout+stderr)}` for every shipped pair."""
    out: dict[str, tuple[int, str]] = {}
    for name, ref, tomls in pairs():
        tree = TESTS if ref is None else _tree_at(ref, cache, workdir)
        source = os.path.join(tree, name)
        if not os.path.exists(source):
            sys.exit(f"SWEEP FAILED — {name} is not present at "
                     f"{ref} either; nothing to audit against")
        done = _run(sys.executable, audit, source, *tomls, cwd=tree)
        key = f"{name}@{'tree' if ref is None else ref[:10]}"
        text = _normalise(done.stdout + done.stderr)
        _require_a_verdict(key, done.returncode, done.stdout, done.stderr)
        out[key] = (done.returncode, text)
    return out


def _require_a_verdict(key: str, rc: int, stdout: str, stderr: str) -> None:
    """A CRASH IS NOT A VERDICT, and this differential could not tell them apart.

    THE HOLE, AND WHAT IT COST. `sweep` recorded `(returncode, output bytes)`
    and `--compare` called any difference in either a "moved verdict". A Python
    traceback is a difference in both -- so a patch that made the audit script
    DIE on every pair read as "185 of 268 verdicts moved", and that figure
    survived a whole batch and was defended in a report as a finding about the
    corpus. It was a finding about the patch. Batch 4's own fix round records
    it: "my differential counted a non-zero exit as a moved verdict and could
    not tell a changed answer from no answer at all -- the same failure this
    project keeps naming in other people's instruments, in mine, and this time
    in the one instrument whose whole job is to notice."

    THE CHECK IS INDEPENDENT OF THE DIFFERENTIAL, which is the point: it does
    not compare two runs, it asks whether THIS run produced an answer at all.
    Two conditions, taken from the audit script's own contract:

      * no traceback text on either stream -- an audit that raised did not
        decide anything;
      * a non-zero exit must be accompanied by an `AUDIT `-prefixed verdict
        line. rc=1 means "AUDIT FAILED", rc=2 means a usage/resolution error
        the script prints in its own words; a bare non-zero with no verdict is
        the script falling over.

    It runs on BOTH sides of a `--compare`, because `sweep` is what both sides
    call, so the pre-change revision is held to it too. That matters: the
    crash in the 185 case was on the CHANGED side, but the symmetric mistake --
    a historical ref that no longer runs under today's Python -- is the same
    defect read backwards.
    """
    both = stdout + stderr
    if "Traceback (most recent call last)" in both:
        sys.exit(
            f"SWEEP FAILED — {key}: the audit script RAISED. That is not a "
            f"verdict, and comparing it as one is how a crash gets reported as "
            f"a moved verdict:\n{both[-2000:]}")
    if rc != 0 and not re.search(r"^AUDIT ", both, re.M):
        sys.exit(
            f"SWEEP FAILED — {key}: exit {rc} with no `AUDIT ...` verdict line. "
            f"The script did not decide; it stopped.\n{both[-2000:]}")


def _audit_at(ref: str, workdir: str) -> str:
    """`scripts/audit-case-migration.py` as of `ref`, on disk."""
    shown = _run("git", "-C", REPO, "show", f"{ref}:{AUDIT_REL}")
    if shown.returncode:
        sys.exit(f"{ref} has no {AUDIT_REL}: {shown.stderr.strip()}")
    path = os.path.join(workdir, f"audit_{ref.replace('/', '_')}.py")
    with open(path, "w") as handle:
        handle.write(shown.stdout)
    return path


def _selftest() -> int:
    """Prove the sweep can fail. A differential that answers "identical"
    unconditionally is worth nothing, and answering that way is exactly what
    it does when it compares a script against itself and nothing else --
    so the poison here is a real behavioural edit to a copy of the audit
    script, applied to the smallest corpus the sweep will accept."""
    problems: list[str] = []

    # 1. The corpus resolves, entirely. `pairs()` exits non-zero rather than
    #    returning a short list, so reaching this line is itself the check.
    found = pairs()
    n_tomls = sum(len(t) for _n, _r, t in found)
    n_files = len(glob.glob(os.path.join(CASES, "*", "*.toml")))
    print(f"  resolution -- {len(found)} source(s) covering {n_tomls} case file(s) "
          f"of {n_files} in the corpus")
    if n_tomls != n_files:
        problems.append(f"resolved {n_tomls} case files but the corpus has {n_files}")

    workdir = tempfile.mkdtemp(prefix="audit_sweep_selftest_")
    try:
        cache: dict[str, str] = {}
        # 2. A KNOWN POSITIVE and a KNOWN NEGATIVE on one real pair, rather
        #    than on the whole corpus (which costs minutes). The negative is
        #    the false-green this instrument was written to measure: an
        #    unreferenced `[constants]` entry carrying a deleted claim's text.
        probe_dir = os.path.join(workdir, "probe")
        os.makedirs(probe_dir)
        rs = os.path.join(TESTS, "nullish_assign_reject.rs")
        toml = os.path.join(CASES, "nullish", "assign_reject.toml")
        if not (os.path.exists(rs) and os.path.exists(toml)):
            problems.append("the nullish_assign_reject probe pair is gone; "
                            "re-point this selftest at a live pair")
        else:
            control = os.path.join(probe_dir, "control.toml")
            shutil.copyfile(toml, control)
            body = open(toml).read()
            claim = 'stderr_contains = ["E5506"]'
            if claim not in body:
                problems.append(f"the probe pair no longer carries {claim}")
            poisoned = os.path.join(probe_dir, "poisoned.toml")
            with open(poisoned, "w") as handle:
                handle.write('[constants]\nUNUSED_NOTE = "E5506"\n\n'
                             + body.replace(claim, ""))
            audit = os.path.join(REPO, AUDIT_REL)
            good = _run(sys.executable, audit, rs, control, cwd=TESTS)
            bad = _run(sys.executable, audit, rs, poisoned, cwd=TESTS)
            print(f"  known positive -- the shipped pair: rc={good.returncode}")
            print(f"  known poison   -- claim deleted, dead `[constants]` "
                  f"carrying its text: rc={bad.returncode}")
            if good.returncode != 0:
                problems.append("the known-positive pair does not audit clean")
            if bad.returncode == 0:
                problems.append("THE FALSE GREEN IS BACK: a dropped claim plus an "
                                "unreferenced [constants] entry audits OK")

        # 3. The differential can report a difference. Compare the working
        #    tree's audit script against a deliberately broken copy of it on
        #    one pair, and require the comparison to notice.
        broken = os.path.join(workdir, "audit_broken.py")
        text = open(os.path.join(REPO, AUDIT_REL)).read()
        needle = 'print("\\nAUDIT OK'
        if needle not in text:
            problems.append("cannot build the differential poison -- the audit "
                            "script's success line has moved")
        else:
            with open(broken, "w") as handle:
                handle.write(text.replace(needle, 'print("\\nAUDIT DIFFERENT', 1))
            here = _run(sys.executable, os.path.join(REPO, AUDIT_REL), rs, toml, cwd=TESTS)
            there = _run(sys.executable, broken, rs, toml, cwd=TESTS)
            differs = _normalise(here.stdout) != _normalise(there.stdout)
            print(f"  differential   -- an edited audit script is seen as different: "
                  f"{differs}")
            if not differs:
                problems.append("the differential cannot see an edited audit script")

            # THE CRASH ARM'S OWN KNOWN POSITIVE. `_require_a_verdict` exists
            # because a traceback used to be read as a moved verdict; a check
            # whose green is indistinguishable from "it cannot fire" would
            # reproduce that failure one level up. Both of its conditions are
            # driven here, against synthetic output, and both must exit.
            for label, rc, text in (
                    ("a traceback", 1,
                     "Traceback (most recent call last)\n  File \"x\"\nBoom"),
                    ("a bare non-zero with no verdict", 1, "some noise\n")):
                caught = False
                try:
                    _require_a_verdict("selftest", rc, text, "")
                except SystemExit:
                    caught = True
                print(f"  crash arm      -- {label} is refused: {caught}")
                if not caught:
                    problems.append(
                        f"_require_a_verdict does not fire on {label}")
            # and it must NOT fire on a real verdict, either direction
            for rc, text in ((0, "AUDIT OK — fine\n"),
                             (1, "AUDIT FAILED — 1 claim(s) absent\n")):
                try:
                    _require_a_verdict("selftest", rc, text, "")
                except SystemExit:
                    problems.append(
                        f"_require_a_verdict fires on a real rc={rc} verdict")
            print("  crash arm      -- a real AUDIT verdict passes: True")
        del cache
    finally:
        shutil.rmtree(workdir, ignore_errors=True)

    if problems:
        print("\nSWEEP SELFTEST FAILED")
        for problem in problems:
            print(f"  {problem}")
        return 1
    print("\nSWEEP SELFTEST OK — every shipped case file resolves to a source, "
          "the known positive is green, the dead-constant poison is caught, and "
          "the differential can see an edited gate")
    return 0


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--compare", metavar="REF",
                        help="also run REF's own audit script over this corpus "
                             "and require identical verdicts")
    parser.add_argument("--selftest", action="store_true")
    args = parser.parse_args(argv)

    if args.selftest:
        return _selftest()

    workdir = tempfile.mkdtemp(prefix="audit_sweep_")
    try:
        cache: dict[str, str] = {}
        here = sweep(os.path.join(REPO, AUDIT_REL), workdir, cache)
        red = {k: v for k, v in here.items() if v[0] != 0}
        print(f"{len(here)} source(s) audited; "
              f"{len(here) - len(red)} exit 0; {len(red)} exit non-zero")
        for key, (rc, out) in sorted(red.items()):
            reason = next((line for line in out.splitlines()
                           if line.startswith("AUDIT FAILED")), "")
            print(f"  rc={rc}  {key}  {reason}")

        if not args.compare:
            print("\nSWEEP DONE — this is a report. Pass --compare REF to gate "
                  "on a verdict differential, or --selftest for the gating arm.")
            return 0

        there = sweep(_audit_at(args.compare, workdir), workdir, cache)
        moved = sorted(set(here) ^ set(there))
        rc_moved = sorted(k for k in set(here) & set(there)
                          if here[k][0] != there[k][0])
        out_moved = sorted(k for k in set(here) & set(there)
                           if here[k][1] != there[k][1])
        print(f"\nvs {args.compare}: {len(moved)} pair(s) present on one side only, "
              f"{len(rc_moved)} return code(s) moved, "
              f"{len(out_moved)} output(s) moved")
        for key in moved:
            print(f"  ONLY ON ONE SIDE  {key}")
        for key in rc_moved:
            print(f"  RC MOVED  {key}: {there[key][0]} -> {here[key][0]}")
        for key in out_moved:
            print(f"  OUTPUT MOVED  {key}")
        if moved or rc_moved or out_moved:
            print("\nSWEEP FAILED — a verdict moved. That is a finding: state what "
                  "moved and why before shipping the gate change.")
            return 1
        print(f"\nSWEEP OK — {len(here)} pair(s), every return code and every byte "
              f"of output identical to {args.compare}")
        return 0
    finally:
        shutil.rmtree(workdir, ignore_errors=True)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
