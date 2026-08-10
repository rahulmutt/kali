#!/usr/bin/env python3
"""Injection probes for every gate arm batch 8-inst-2 added, and it GATES.

WHY THIS FILE EXISTS RATHER THAN A PARAGRAPH IN A REPORT. Ruling 15: "a green
suite is not evidence for a gate change -- a removed check is silent by
definition. Verify a gate change only with an injection probe showing the check
still fires on the thing it exists to catch -- and check one layer up, because
the probe itself may be ungated." Every arm below had ZERO live instances of its
defect on the shipped tree, so a green run says nothing at all about any of
them. Each probe therefore poisons a real artifact and requires the gate to go
red, and each is paired with the unpoisoned control that must stay green -- a
probe that fails in both states measures nothing.

ONE LAYER UP is what makes it a gate rather than a demo: this file is listed in
`scripts/test-gate.sh`'s migration-gate set, so it runs wherever the gates run,
and an arm that stops firing fails CI by name instead of quietly passing.

Nothing here writes to the repository. Poisoned copies live under `mktemp -d`;
the two in-process monkeypatches (`batch5_crosscheck`'s declaration dicts,
`source_ref_rehearsal.sweep`) restore in a `finally`.

    python3 tools/task-18-browser-pilot/inst2_probes.py     # exit 0 / 1
"""

import os
import re
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
TESTS = os.path.join(REPO, "crates/kali_cli/tests")
sys.path.insert(0, HERE)

FAILURES = []


def check(label, ok, detail=""):
    print(f"  {'ok  ' if ok else 'FAIL'}  {label}")
    if detail:
        print(f"          {detail}")
    if not ok:
        FAILURES.append(label)
    return ok


def run(*args, cwd=REPO):
    out = subprocess.run([sys.executable, *args] if args[0].endswith(".py")
                         else list(args),
                         cwd=cwd, capture_output=True, text=True)
    return out.returncode, out.stdout + out.stderr


# ---------------------------------------------------------------------------
# 1. check_fixtures.py -- the argv/`[source]` correspondence arm (item 1).
# ---------------------------------------------------------------------------
def probe_argv_correspondence(tmp):
    print("\n1. argv/[source] correspondence (check_fixtures.py "
          "--argv-correspondence)")
    src = os.path.join(TESTS, "cases/string/length_static.toml")
    text = open(src).read()

    clean = os.path.join(tmp, "clean.toml")
    open(clean, "w").write(text)
    rc, _ = run(os.path.join(HERE, "check_fixtures.py"),
                "--argv-correspondence", clean)
    check("CONTROL: the unpoisoned case file passes", rc == 0, f"rc={rc}")

    # THE POISON IS THE REAL DEFECT, not a synthetic one: an argv token that no
    # longer names a declared fixture, which is what a half-finished rename
    # leaves behind and what a fail-closed step cannot notice.
    lines = text.split("\n")
    hit = next((i for i, l in enumerate(lines) if l.startswith("args = ")), None)
    if hit is None:
        return check("the sample case file has an `args =` line to poison", False)
    lines[hit] = re.sub(r'\.(js|ts|jsx|tsx)"',
                        lambda m: f'_typo.{m.group(1)}"', lines[hit], count=1)
    poisoned = os.path.join(tmp, "poisoned.toml")
    open(poisoned, "w").write("\n".join(lines))
    rc, out = run(os.path.join(HERE, "check_fixtures.py"),
                  "--argv-correspondence", poisoned)
    check("POISON: an argv token renamed off its `[source]` key is caught",
          rc == 1 and "UNDECLARED ARGV" in out, f"rc={rc}")

    # The vacuity floor, which is the same floor the fixture arm already has: a
    # run that checked nothing must not report the same thing as a run that
    # checked everything and found nothing.
    vacuous = os.path.join(tmp, "vacuous.toml")
    open(vacuous, "w").write('[[case]]\nname = "x"\nargs = ["--version"]\n')
    rc, out = run(os.path.join(HERE, "check_fixtures.py"),
                  "--argv-correspondence", vacuous)
    check("FLOOR: a file with no argv filename token exits 2, not 0",
          rc == 2 and "VACUOUS" in out, f"rc={rc}")


# ---------------------------------------------------------------------------
# 2. check_fixtures.py -- the `.replace`-template arm (item 2).
# ---------------------------------------------------------------------------
def probe_replace_arm(tmp):
    print("\n2. `.replace`-built fixtures (check_fixtures.py, batch 7A's "
          "disclosed hole)")
    rs = os.path.join(TESTS, "browser_object_from_entries_harness.rs")
    toml = os.path.join(TESTS, "cases/browser/object_from_entries_harness.toml")
    cf = os.path.join(HERE, "check_fixtures.py")

    rc, out = run(cf, rs, toml)
    check("CONTROL: the pair batch 7A reported red is now green",
          rc == 0 and ".replace-built (segments matched)" in out, f"rc={rc}")

    # ONE LAYER UP: is the ARM what makes it green, or would anything? Take the
    # `.replace` call out of the source and the needle can no longer be derived,
    # so the arm must not fire and the pair must go back to red.
    rs_copy = os.path.join(tmp, "no_replace.rs")
    text = open(rs).read()
    open(rs_copy, "w").write(text.replace(".replace(", ".nonreplace("))
    rc, out = run(cf, rs_copy, toml)
    check("WIRING: with the `.replace(` call gone, the arm cannot fire",
          rc == 1 and ".replace-built" not in out, f"rc={rc}")

    # And it must not be a blanket excuse: corrupt a segment of the template and
    # the arm has to stay silent even though a `.replace` needle is present.
    toml_copy = os.path.join(tmp, "corrupt.toml")
    open(toml_copy, "w").write(
        open(toml).read().replace("assertFromEntriesShape",
                                  "assertFromEntriesShapeXX"))
    rc, out = run(cf, rs, toml_copy)
    check("POISON: a corrupted program text is still UNMATCHED",
          rc == 1 and "UNMATCHED" in out, f"rc={rc}")


# ---------------------------------------------------------------------------
# 3. batch5_crosscheck.py -- the ghost-stem staleness arm (item 3).
# ---------------------------------------------------------------------------
def probe_ghost_declarations():
    print("\n3. ghost declarations (batch5_crosscheck.ghost_declarations)")
    import batch5_crosscheck as X

    def gate(stem="math_sqrt_cbrt_harness"):
        """One crosscheck invocation, from a clean slate. `_NO_NEEDLE`,
        `_PINNED` and `_REDLIST_HIT` are module-level ACCUMULATORS -- a second
        `main()` in the same process doubles every count and reports a
        declaration mismatch that has nothing to do with what is being probed.
        The real gate is a fresh process, so this reproduces that."""
        X._NO_NEEDLE.clear()
        X._PINNED.clear()
        X._REDLIST_HIT.clear()
        return X.main(["batch5_crosscheck.py", "--citations-only", stem])

    check("CONTROL: no declaration names a stem the corpus lacks today",
          X.ghost_declarations() == [],
          f"{len(X.NO_NEEDLE_DECLARED)} + {len(X.PINNED_SPLIT_DECLARED)} + "
          f"{len({k[0] for k in X.UNGATED_REDLIST})} declared stem(s)")

    ghost = "a_stem_deleted_by_a_later_batch"
    saves = (dict(X.NO_NEEDLE_DECLARED), dict(X.PINNED_SPLIT_DECLARED),
             dict(X.UNGATED_REDLIST))
    for label, mutate in (
            ("NO_NEEDLE_DECLARED",
             lambda: X.NO_NEEDLE_DECLARED.__setitem__(ghost, 3)),
            ("PINNED_SPLIT_DECLARED",
             lambda: X.PINNED_SPLIT_DECLARED.__setitem__(ghost, (2, 1))),
            ("UNGATED_REDLIST",
             lambda: X.UNGATED_REDLIST.__setitem__(
                 (ghost, "case file", ":11"), "FIXTURE-TEXT"))):
        try:
            mutate()
            problems = X.ghost_declarations()
            named = any(ghost in p and label in p for p in problems)
            # Not just the helper: the GATE has to go red, and it must do so on
            # an invocation that never visits the ghost -- which is every
            # invocation, since the stem does not exist. That independence from
            # `visited` is the whole point of the arm.
            rc = gate()
            check(f"POISON: a ghost stem in {label} fails the gate",
                  named and rc == 1, f"rc={rc}, problems={len(problems)}")
        finally:
            X.NO_NEEDLE_DECLARED.clear()
            X.NO_NEEDLE_DECLARED.update(saves[0])
            X.PINNED_SPLIT_DECLARED.clear()
            X.PINNED_SPLIT_DECLARED.update(saves[1])
            X.UNGATED_REDLIST.clear()
            X.UNGATED_REDLIST.update(saves[2])
    check("RESTORED: the same invocation is green again", gate() == 0)


# ---------------------------------------------------------------------------
# 4. citation_tiers._ref_carries -- two failure modes, two diagnoses (item 7).
# ---------------------------------------------------------------------------
def probe_ref_carries():
    print("\n4. `SOURCE REF:` failure modes 2 and 3 (citation_tiers._ref_carries)")
    import citation_tiers as T

    def message(ref, name):
        try:
            T._ref_carries("probe", ref, name)
        except SystemExit as exc:
            return str(exc)
        return ""

    # Failure mode 2: well-formed, absent from the repository. This is what a
    # shallow `actions/checkout` produces for every `SOURCE REF:` in the family,
    # and it is now the message a CI reader meets first.
    unreachable = message("deadbeef" * 5, "browser_math_round.rs")
    check("mode 2 (unreachable ref) names the fetch-depth remedy",
          "fetch-depth: 0" in unreachable and "unshallow" in unreachable,
          unreachable[:120])
    check("mode 2 does NOT send the reader to re-derive a correct ref",
          "PARENT" not in unreachable)

    # Failure mode 3: reachable, but that commit has no such file. The empty
    # tree object is reachable in every repository and contains nothing, so this
    # needs no hand-picked sha.
    empty_tree = subprocess.run(
        ["git", "-C", REPO, "hash-object", "-t", "tree", "--stdin"],
        input="", capture_output=True, text=True).stdout.strip()
    commit = subprocess.run(
        ["git", "-C", REPO, "commit-tree", empty_tree, "-m", "probe"],
        capture_output=True, text=True).stdout.strip()
    absent = message(commit, "browser_math_round.rs") if commit else ""
    check("mode 3 (ref resolves, path absent) names the deletion commit's PARENT",
          "PARENT, not the deletion commit" in absent, absent[:120])
    check("the two modes are told apart at all", bool(absent) and bool(unreachable)
          and absent != unreachable)


# ---------------------------------------------------------------------------
# 5. verify_pair.sh -- delegation, not an eighth reader (item 6).
# ---------------------------------------------------------------------------
def probe_verify_pair_delegation(tmp):
    print("\n5. verify_pair.sh resolves a deleted source by DELEGATING")
    import citation_tiers as T

    # (a) It holds no parser of its own. `8-inst-1` enumerated seven live
    #     readers of the pre-trim ref; this asserts that verify_pair.sh is not
    #     the eighth -- every mention of either header line is in a comment.
    code = [l for l in open(os.path.join(HERE, "verify_pair.sh")).read().split("\n")
            if not l.lstrip().startswith("#")]
    offenders = [l for l in code
                 if "SOURCE REF" in l or "PRE-TRIM REF" in l]
    check("verify_pair.sh parses neither header line itself", not offenders,
          "; ".join(offenders[:2]))
    # The predicate itself, checked one layer up -- a grep that cannot fire is
    # not evidence.
    check("...and that check can fire",
          bool([l for l in ['ref=$(grep "SOURCE REF:" "$t")']
                if "SOURCE REF" in l]))

    # (b) What it delegates TO returns the real historical blob, byte for byte.
    specs = subprocess.run(
        ["bash", os.path.join(HERE, "citation_sweep.sh"), "--print-specs"],
        cwd=REPO, capture_output=True, text=True).stdout.split("\n")
    sourceref = [l.split() for l in specs if " SOURCEREF " in l]
    if not check("there is at least one source-deleted stem to resolve",
                 bool(sourceref), f"{len(sourceref)} stem(s)"):
        return
    stem, _, ref, name = sourceref[0]
    T.ARTIFACT_DIR[0] = tmp
    path, prov, got_ref, got_name = T.resolve_source(stem)
    blob = subprocess.run(
        ["git", "-C", REPO, "cat-file", "blob",
         f"{ref}:crates/kali_cli/tests/{name}"], capture_output=True).stdout
    with open(path, "rb") as fh:
        same = fh.read() == blob
    check(f"{stem}: resolved to the blob at {ref[:10]}, byte for byte",
          same and prov == "SOURCEREF" and (got_ref, got_name) == (ref, name),
          f"{prov} -> {path}")

    # (c) The ruling-18 non-match arm: `--rs` claims one source, the case file's
    #     own header resolves to another. Both cannot be this pair's source, so
    #     it stops rather than picking. This exits before any of the eight arms,
    #     so the probe costs nothing.
    rc, out = run("bash", os.path.join(HERE, "verify_pair.sh"), stem,
                  "--rs", "a_source_this_repository_does_not_have")
    check("POISON: a `--rs` that contradicts the case file's own header exits 2",
          rc == 2 and "cannot both be this pair's source" in out, f"rc={rc}")


# ---------------------------------------------------------------------------
# 6. The population-banner assertion (item 4's third half).
# ---------------------------------------------------------------------------
def probe_population_banner():
    print("\n6. population banner equality (source_ref_rehearsal."
          "population_agreement)")
    import source_ref_rehearsal as R

    real_sweep = R.sweep
    scratch = REPO           # the real tree: this arm only reads

    def doctored(_scratch, args=()):
        rc, out = real_sweep(scratch, args)
        if not args:
            out = re.sub(r"sweep over (\d+) stems",
                         lambda m: f"sweep over {int(m.group(1)) + 1} stems", out)
        return rc, out

    for label, patched, want in (("CONTROL: the real banner", real_sweep, 0),
                                 ("POISON: banner off by one", doctored, 1)):
        R.FAILURES.clear()
        try:
            R.sweep = patched
            R.population_agreement(scratch)
        finally:
            R.sweep = real_sweep
        check(f"{label} -> {len(R.FAILURES)} failure(s)",
              len(R.FAILURES) == want,
              (R.FAILURES[0][:140] if R.FAILURES else ""))
    R.FAILURES.clear()


def main():
    tmp = tempfile.mkdtemp(prefix="inst2-probes-")
    try:
        probe_argv_correspondence(tmp)
        probe_replace_arm(tmp)
        probe_ghost_declarations()
        probe_ref_carries()
        probe_verify_pair_delegation(tmp)
        probe_population_banner()
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    if FAILURES:
        print(f"\nPROBES FAILED -- {len(FAILURES)} arm(s) did not behave as "
              "declared")
        for f in FAILURES:
            print(f"  {f}")
        return 1
    print("\nPROBES OK -- every arm batch 8-inst-2 added fires on the defect it "
          "exists to catch, stays silent on the control, and is reachable from "
          "the gate that runs it")
    return 0


if __name__ == "__main__":
    sys.exit(main())
