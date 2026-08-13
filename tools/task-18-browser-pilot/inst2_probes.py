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
`scripts/test-gate.sh --gates-only`'s migration-gate set, so it runs wherever
the gates run, and an arm that stops firing fails CI by name instead of quietly
passing.

WHAT IS PROBED, ENUMERATED -- and this list is the claim, in place of the
universal quantifier that used to close `main()` and was false (round 1, I1):

  1. `check_fixtures.argv_source_correspondence` -- the arm, and its vacuity floor
  2. `check_fixtures`'s `.replace`-template arm -- that it fires, that removing
     the `.replace(` call unfires it, and that it is not a blanket excuse
  3. `batch5_crosscheck.ghost_declarations` -- all three declaration tables,
     through `main()` rather than through the helper
  4. `citation_tiers._ref_carries` -- failure modes 2 and 3, and that they differ
  5. `verify_pair.sh` -- that it holds no reader of either ref header, that what
     it delegates to returns the real blob, and its `--rs` non-match arm
  6. `source_ref_rehearsal.population_agreement` -- the banner limb and the
     printed-line-count limb
  7. `check_fixtures._kind_of` (the predicate that decides the whole population),
     `check_fixtures.main`'s chained `return argv_main(...)`, both `${...}`
     substitution failure paths, and `citation_tiers.resolve_source`'s
     no-case-file exit
  8. `batch5_crosscheck.check`'s NO-SOURCE branch -- that it no longer lets the
     banner claim "header structure consistent" over a population that branch
     returned before checking, that it does not call a U2 split stem's source
     "deleted", and that the disclosure vanishes when the line recording it is
     removed (added batch 8A fix round 3; this is the second limb of the defect
     fix round 1 closed for `--citations-only`)

WHAT IS NOT, named rather than left to be assumed: `verify_pair.sh`'s two
resolver-failure branches (`cannot resolve a source`, `resolver returned a
non-file`) -- neither is reachable without mutating a shipped case file, which
this script may not do; and `population_agreement`'s `len(declared) != 1` guard,
which needs a doctored `--print-specs` shape rather than a doctored figure.

Nothing here writes to the repository PERMANENTLY -- no object, no config, and
no shipped case file at all. ONE EXCEPTION, stated exactly rather than by
analogy: probe 8 rewrites the REAL `tools/task-18-browser-pilot/
batch5_crosscheck.py` in place for one subprocess call and restores it in a
`finally`. If the process is killed between the two, the working tree is left
with a mutated gate.

That is NOT what `source_ref_rehearsal.selftest_kill_power` does, and an earlier
version of this paragraph claimed it was. That probe mutates its copy inside a
throwaway `git clone --shared` under `mkdtemp` (:105-108) and so CANNOT dirty
the real tree under any failure. The clone-based pattern is strictly safer than
what probe 8 does; probe 8 is not an instance of it. Matching it here is
feasible -- the gate resolves its own paths from `__file__`, so a checked-out
clone would work -- at the cost of a full checkout per run. Deliberately left as
a disclosed risk rather than restructured unasked. Round 1
(C1) had this sentence while probe 4 used `git commit-tree`, which needs a
committer identity CI does not have AND leaves a dangling object every run; it
now derives a real ancestor commit instead. Poisoned copies live under
`mktemp -d`; the in-process monkeypatches (`batch5_crosscheck`'s declaration
dicts, `source_ref_rehearsal.sweep`, `check_fixtures._kind_of`) restore in a
`finally`.

    python3 tools/task-18-browser-pilot/inst2_probes.py     # exit 0 / 1

    # the CI-equivalent control -- no user identity, no global config:
    GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null \
        python3 tools/task-18-browser-pilot/inst2_probes.py
"""

import contextlib
import io
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

# EVERY PROBE BELOW THAT NAMES A `browser_*.rs` NAMES ONE 8C DELETED.
#
# These probes hand a source PATH to `check_fixtures.py`, so they need the file
# on disk, and after the family deletion `os.path.join(TESTS, name)` is a path
# to nothing. That did not make one probe go red -- it raised
# `FileNotFoundError` out of `main()` and took every probe after it down with
# it, which is the failure mode ruling 15 warns about wearing its worst face: a
# suite that stops running looks, from the exit code alone, exactly like a suite
# that found one problem.
#
# `case_emit.source_bytes` reads the working tree first and the declared
# family-deletion ref otherwise, so this is the same one resolver the generators
# and the sweep use, and the probes keep running against the exact bytes they
# were written against.
_SRC_DIR = [None]


def source_path(name):
    """`<a real file holding browser_<...>.rs>`, from the tree or from history.

    THE `#[path]` SIBLINGS COME TOO, and leaving them out is not a smaller
    version of the same thing. `browser_runtime_summary_fallback_ts_input.rs` is
    a U10 carrier whose `env`-carried program lives in `run.rs`/`test.rs`; a
    carrier materialised alone has `#[path]` declarations resolving to nothing,
    so `check_fixtures` finds no fixture, reports its vacuity floor, and the
    probe that corrupts one byte of that program sees no change. The arm did not
    go red and did not go green -- it stopped having a subject, which is the
    silent-arm failure ruling 18 is about.
    """
    from case_emit import source_bytes  # noqa: E402  (late: cycle-free)

    live = os.path.join(TESTS, name)
    if os.path.exists(live):
        return live
    if _SRC_DIR[0] is None:
        _SRC_DIR[0] = tempfile.mkdtemp(prefix="inst2-probe-sources-")
    out = os.path.join(_SRC_DIR[0], name)
    if not os.path.exists(out):
        text = source_bytes(name)
        with open(out, "w") as fh:
            fh.write(text)
        for sub in re.findall(r'#\[path = "([^"]+)"\]', text):
            dst = os.path.join(_SRC_DIR[0], sub)
            os.makedirs(os.path.dirname(dst), exist_ok=True)
            with open(dst, "w") as fh:
                fh.write(source_bytes(sub))
    return out


_SPEC_DIR = [None]


def spec(stem):
    """`<stem>`, or `<stem>=<path>` once the source is out of the tree.

    THE SAME OVERRIDE `verify_pair.sh` PASSES, for the reason its own comment
    gives: a bare stem with no `browser_<stem>.rs` falls into
    `batch5_crosscheck`'s no-source branch and runs the GATEDNESS arm alone --
    green, and reading none of the pair's citations. A probe whose CONTROL takes
    that branch is asserting something about a code path it did not mean to
    exercise, which after 8C's family deletion was every migrated stem.

    Resolution goes through `citation_tiers.resolve_source`, so the probes ask
    the same resolver the sweep does rather than becoming a second opinion about
    which blob a stem's citations mean.
    """
    import citation_tiers as C

    if os.path.exists(os.path.join(TESTS, f"browser_{stem}.rs")):
        return stem
    if _SPEC_DIR[0] is None:
        _SPEC_DIR[0] = tempfile.mkdtemp(prefix="inst2-probe-specs-")
        C.ARTIFACT_DIR[0] = _SPEC_DIR[0]
    path, _prov, _ref, _name = C.resolve_source(stem)
    return f"{stem}={path}"


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
    # THE SOURCE IS GONE FROM THE TREE (8C's family deletion) and every arm
    # below needs it as a FILE on disk to hand to `check_fixtures.py`. Reading
    # it out of the tree raised `FileNotFoundError` and took the whole probe
    # suite down with it -- including the arms after this one, which is worse
    # than the arm merely going red. Materialise it once, from the same
    # family-deletion ref the case file declares, and let the arms run against
    # exactly the bytes they were written for.
    rs = source_path("browser_object_from_entries_harness.rs")
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
        # `spec()`, not the bare stem: this stem's source left the tree with the
        # family, and a bare stem now resolves nothing, so the "green again"
        # control would compare one no-source run against another.
        return X.main(["batch5_crosscheck.py", "--citations-only", spec(stem)])

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

    # Failure mode 3: reachable, but that commit has no such file. DERIVED FROM
    # HISTORY, not synthesised: the parent of the commit that ADDED the source is
    # a real ancestor that predates it, which is the same shape as naming the
    # deletion commit instead of its parent.
    #
    # ROUND 1 (C1): this used `git commit-tree` over the empty tree. That needs a
    # committer identity, which `actions/checkout` does not set and a GitHub
    # runner's hostname cannot supply a fallback domain for, so `git` fatalled and
    # BOTH mode-3 checks failed -- on the one machine this whole dispatch exists
    # to wire the gates into. It was invisible locally because the PROCESS
    # inherits ~/.gitconfig through $HOME while a fresh `git clone` inherits
    # nothing, so a clean-checkout run could not catch it. Reproduce the fixed
    # state under the same conditions CI has:
    #
    #     GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null \
    #         python3 tools/task-18-browser-pilot/inst2_probes.py
    #
    # It also left a dangling object in .git/objects on every run, which is why
    # this file's "writes nothing" claim was false. Nothing is written now.
    name = "browser_math_round.rs"
    path = f"crates/kali_cli/tests/{name}"
    added = subprocess.run(
        ["git", "-C", REPO, "log", "--diff-filter=A", "-1", "--format=%H",
         "--", path], capture_output=True, text=True).stdout.strip()
    parent = subprocess.run(
        ["git", "-C", REPO, "rev-parse", "-q", "--verify", f"{added}^"],
        capture_output=True, text=True).stdout.strip() if added else ""
    if not check(f"a real ancestor predating {name} could be derived",
                 len(parent) == 40, f"added in {added[:10]}, parent {parent[:10]}"):
        return
    absent = message(parent, name)
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
    def header_readers(lines):
        """Non-comment lines that read either ref header. TWO halves, and the
        control below runs BOTH: the comment filter and the substring test."""
        return [l for l in lines
                if not l.lstrip().startswith("#")
                and ("SOURCE REF" in l or "PRE-TRIM REF" in l)]

    text = open(os.path.join(HERE, "verify_pair.sh")).read()
    offenders = header_readers(text.split("\n"))
    check("verify_pair.sh parses neither header line itself", not offenders,
          "; ".join(offenders[:2]))
    # ONE LAYER UP, THROUGH THE REAL PREDICATE (round 1, I2). The previous
    # version re-tested the substring half against a synthetic literal with an
    # inline comprehension, so inverting the comment filter -- the exact way this
    # check silently stops working, since verify_pair.sh's own prose mentions
    # both headers a dozen times -- emptied `offenders` while "can fire" stayed
    # green. The synthetic file now goes through `header_readers` itself, and
    # carries a commented mention that must NOT be returned alongside the real
    # reader that must.
    synthetic = ['#   a comment mentioning PRE-TRIM REF: and SOURCE REF: in prose',
                 '  # an indented comment mentioning SOURCE REF: too',
                 'ref=$(grep -oP "(?<=SOURCE REF:)\\s*\\S+" "$t")',
                 'echo hello']
    fired = header_readers(synthetic)
    check("...and that predicate fires on a real reader",
          fired == [synthetic[2]], f"returned {fired!r}")

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

    def dropped_spec(_scratch, args=()):
        """The OTHER limb (round 1, I1): the printed listing loses a stem while
        `#population` and the banner still agree. That is a spec appended to one
        of the sweep's two arrays and not the other -- the failure the
        three-way comparison exists for, and the one the banner limb cannot see."""
        rc, out = real_sweep(scratch, args)
        if args:
            lines = out.split("\n")
            first = next(i for i, l in enumerate(lines)
                         if l.strip() and not l.startswith("#"))
            out = "\n".join(lines[:first] + lines[first + 1:])
        return rc, out

    for label, patched, want in (("CONTROL: the real banner", real_sweep, 0),
                                 ("POISON: banner off by one", doctored, 1),
                                 ("POISON: one spec line dropped from the "
                                  "listing", dropped_spec, 2)):
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


# ---------------------------------------------------------------------------
# 7. The arms round 1 (I1) named as added-but-unprobed.
# ---------------------------------------------------------------------------
def probe_supporting_arms(tmp):
    print("\n7. the supporting arms (round 1, I1)")
    import check_fixtures as C
    import citation_tiers as T

    # `_kind_of` DECIDES THE WHOLE POPULATION. A wrong answer does not fail; it
    # silently shrinks the 4403 steps the gate looks at, which is the direction
    # nothing else notices.
    check("_kind_of: a bare step is `cli` (model.rs's default)",
          C._kind_of({"args": ["run", "x.js"]}) == "cli")
    check("_kind_of: an explicit kind wins",
          C._kind_of({"kind": "file_json", "path": "p"}) == "file_json")
    check("_kind_of: a kind-specific field is NOT silently `cli`",
          all(C._kind_of({f: "v"}) != "cli"
              for f in ("path", "fields", "entry", "body")),
          "path/fields/entry/body")
    # One layer up: collapse the predicate and the floor must catch it, because
    # a population that has quietly gone to zero is the failure mode.
    real_kind_of = C._kind_of
    try:
        C._kind_of = lambda step: "file_json"
        rc, out = None, None
        rc = C.argv_main([os.path.join(TESTS, "cases/string/length_static.toml")])
    finally:
        C._kind_of = real_kind_of
    check("_kind_of collapsed -> the vacuity floor fires (rc 2, not 0)", rc == 2,
          f"rc={rc}")

    # `check_fixtures.main`'s chained `return argv_main(...)` -- the whole reason
    # it was chained. A pair that is GREEN on fixtures and RED on argv must exit
    # non-zero; before the chain it returned 0 at the fixture arm.
    rs = source_path("browser_object_from_entries_harness.rs")
    src = os.path.join(TESTS, "cases/browser/object_from_entries_harness.toml")
    chained = os.path.join(tmp, "chained.toml")
    open(chained, "w").write(
        open(src).read().replace('"main.js"]', '"main_typo.js"]', 1))
    rc, out = run(os.path.join(HERE, "check_fixtures.py"), rs, chained)
    check("chained return: fixtures green + argv red still exits non-zero",
          rc == 1 and "FIXTURE CHECK OK" in out and "UNDECLARED ARGV" in out,
          f"rc={rc}")

    # The two `${...}` substitution failure paths, in argv and in a `[source]`
    # key. Both are reported as problems rather than crashing, and -- since round
    # 1 -- rank above the vacuity floor even though they yield no token.
    for label, body in (
            ("argv", '[[case]]\nname = "x"\nargs = ["run", "${nope}.js"]\n'),
            ("[source] key", '[source]\n"${nope}.js" = "x"\n\n[[case]]\n'
                             'name = "x"\nargs = ["run", "a.js"]\n')):
        bad = os.path.join(tmp, f"unresolved_{label.split()[0].strip('[')}.toml")
        open(bad, "w").write(body)
        rc, out = run(os.path.join(HERE, "check_fixtures.py"),
                      "--argv-correspondence", bad)
        check(f"unresolved `${{...}}` in {label} is a reported problem, rc 1",
              rc == 1 and "unresolved placeholder" in out, f"rc={rc}")

    # `resolve_source`'s no-case-file exit: it answers for a MIGRATED pair and
    # must say so rather than guessing.
    try:
        T.resolve_source("a_stem_with_no_case_file")
        raised = ""
    except SystemExit as exc:
        raised = str(exc)
    check("resolve_source refuses a stem with no case file",
          "answers for a MIGRATED pair" in raised, raised[:110])


# ---------------------------------------------------------------------------
# 8. The no-source branch must not let the banner claim an arm it skipped
#    (batch 8A fix round 3).
# ---------------------------------------------------------------------------
def probe_structure_arm_disclosure(tmp):
    """`batch5_crosscheck.py <stem>` used to print `CROSSCHECK OK — header
    structure consistent` for a stem whose source it never resolved.

    `check()`'s no-source branch returns before the structure arm, so a caller
    who did NOT pass `--citations-only` -- i.e. one who believes section order
    is being checked -- got a clean verdict over a population the arm never
    touched. Fix round 1 closed the `--citations-only` limb of the same defect;
    this is the other limb, and it is the one 8C meets, because a U2 split stem
    and a `SOURCE REF:` stem both land here.

    Measured cost of the silence: a per-stem census over `cases/browser/`
    reports 44 stems failing structure where the sweep's own materialised route
    reports 67, and the 23-stem gap is exactly this branch.

    THREE LIMBS, and the third is the one that makes the first two evidence:
      * CONTROL -- a stem WITH a source still claims "header structure
        consistent", so the disclosure has not simply been switched on for
        everything;
      * POISON -- a split stem discloses the skip and does NOT claim structure
        was checked;
      * KILL POWER -- delete the line that records the skip and the disclosure
        must vanish. A probe that stays red when the thing it guards is removed
        is testing something else.

    The kill-power limb asserts on the DISCLOSURE, not on the "header structure
    consistent" phrase, and that is deliberate: a split stem also trips the
    no-needle equality, so it takes the CROSSCHECK FAILED path and never reaches
    the OK banner at all. Asserting the phrase reappears would be asserting
    something neither state prints. The disclosure is printed on BOTH verdicts,
    which is what makes it observable here.
    """
    print("\n8. the no-source branch's banner (batch 8A fix round 3)")
    path = os.path.join(REPO, "tools/task-18-browser-pilot/batch5_crosscheck.py")
    pristine = open(path).read()

    # The recorded skip is what the banner reads; deleting it is the mutation.
    guarded = "        _STRUCTURE_SKIPPED[stem] = why\n"
    if pristine.count(guarded) != 1:
        check("locate the line the disclosure rests on", False,
              "expected exactly one `_STRUCTURE_SKIPPED[stem] = why`")
        return

    # RE-BASED BY 8C ONTO THE SUB-BRANCHES THAT STILL EXIST.
    #
    # This branch has three outcomes: a resolvable source (normal arms run), a
    # named source in the tree that is a U4 TRIM (hint, warned against), and a
    # named source that is gone (no hint, because there is no override to
    # offer). The family deletion moved two of the three subjects:
    #
    #   * `promise_all_bundle` was the "sourced" control BECAUSE
    #     `browser_promise_all_bundle.rs` was in the tree. It is not any more, so
    #     the control took the very branch it exists to contrast with. It is
    #     still the right control -- through `spec()`, which is how the sweep
    #     passes it -- because a resolved override is what "sourced" means now.
    #   * `non_literal_iterator_sources_explicit_api` was the "U2 split whose
    #     source is in the tree and is NOT trimmed" case, and NO stem in the
    #     shipped corpus has that shape any more: every U2 split's source went
    #     with the family, and the only named source left in the tree is
    #     `browser_reflect_own_keys.rs`, which IS trimmed. Rather than delete the
    #     arm (a probe that stops firing is the thing this file exists to
    #     prevent), it is re-based onto what that stem now legitimately IS -- the
    #     source-deleted case -- and asserts the property that case must have:
    #     it offers NO override string, because there is no file to point at.
    sourced = "promise_all_bundle"           # resolved through spec()
    split = "reflect_own_keys_explicit_api"  # U2 split of a U4-TRIMMED carrier
    deleted_split = "non_literal_iterator_sources_explicit_api"  # source now gone

    def banner(stem):
        _rc, out = run(path, stem)
        return out

    control = banner(spec(sourced))
    check("CONTROL: a sourced stem still claims structure was checked",
          "header structure consistent" in control,
          control[-200:])

    poisoned = banner(split)
    check("POISON: a split stem does NOT claim structure was checked",
          "header structure consistent" not in poisoned, poisoned[-200:])
    check("POISON: and it says so, naming the stem and why",
          "header structure checked for" in poisoned and split in poisoned,
          poisoned[-300:])
    check("POISON: and it does not call a split stem's source deleted",
          "source deleted post-migration" not in poisoned, poisoned[-300:])

    # THE HINTS ARE OBEYED, NOT READ. A hint that breaks when followed is worse
    # than no hint, and this branch shipped one for exactly one round: for the
    # TRIMMED carrier's split it recommended `<stem>=<tree path>`, which
    # resolves against the trimmed file and reports spurious `past end of the
    # source` failures. Checking the wording would not have caught that; running
    # it does.
    # THE EXTRACTION IS ASSERTED FOR BOTH ARMS, BEFORE EITHER BRANCHES.
    # Round 4 shipped this loop with the `must_work=True` arm asserting
    # `m is not None` and the `else` arm guarding on a bare `if m:`. So a regex
    # that stopped matching -- a reformatted hint, backticks dropped, wording
    # otherwise intact -- silently skipped the truth-check and `PROBES OK` still
    # printed. That is ruling 18's exact failure mode, "failure to match is
    # indistinguishable from nothing to check", inside the code added to guard
    # against that class. Hoisting the check makes a non-match LOUD for every
    # arm, present and future, rather than for the one that remembered to ask.
    import re as _re
    # `want_override` is asserted in BOTH directions, so a hint that grows an
    # override string it cannot honour is as loud as one that loses the string
    # it should have. That two-sided form is what keeps the re-based arm a probe
    # rather than a description of today's output.
    for stem, want_override in ((deleted_split, False), (split, True)):
        text = banner(stem)
        m = _re.search(r"`([a-z0-9_]+=crates/kali_cli/tests/[^`]+)`", text)
        check(f"OVERRIDE STRING {'EXTRACTED from' if want_override else 'ABSENT from'}"
              f" {stem}'s hint (a miss here would silently skip the limbs below)",
              (m is not None) == want_override, text[:220])
        if not want_override:
            # The source-deleted case: there is no file to point at, so the
            # branch must say so plainly and offer nothing to obey.
            check(f"DELETED SOURCE: {stem} is named as deleted, not as a split "
                  "with a followable override",
                  "source deleted post-migration" in text, text[:220])
            continue
        if m is None:
            continue
        # The trimmed carrier: the same form must be WARNED AGAINST, and the
        # warning must be true -- so obey it anyway and require it to fail.
        check(f"HINT WARNS: {stem} says that override would resolve against "
              "the TRIMMED file",
              "would resolve against the TRIMMED file" in text, text[:200])
        _rc, out = run(path, m.group(1))
        check("HINT WARNS: and the warning is true -- obeying it does "
              "produce a past-end failure",
              "past end of the source" in out, out[-200:])

    try:
        open(path, "w").write(pristine.replace(guarded, "        pass\n"))
        killed = banner(split)
        check("KILL POWER: with the recording deleted, the disclosure vanishes "
              "and the reader is told nothing",
              "header structure checked for" not in killed, killed[-200:])
    finally:
        open(path, "w").write(pristine)
    restored = banner(split)
    check("restored: the disclosure is back",
          "header structure checked for" in restored, restored[-200:])


# ---------------------------------------------------------------------------
# 9. `check_fixtures` must see a program carried in a step's `env`
#    (batch 8B).
# ---------------------------------------------------------------------------
def probe_env_program_texts(tmp):
    """Batch 8B taught `toml_program_texts` to read a step's `env` VALUES.

    Until then `[source]` and a `browser_bundle_harness` `body` were the only
    two places a migrated program could live. The four
    `runtime_summary_fallback_*` targets carry a whole `node -e '...'` script --
    the thing that fabricates the browser harness's summary file and stdout --
    in `KALI_BROWSER_BUNDLE_HARNESS_COMMAND`, per case, because that is where
    the source puts it.

    The loud direction (46 correct fixtures reported UNMATCHED) is not what
    justifies the change; a false failure is at least visible. The QUIET
    direction is: before the change the arm could not distinguish a faithful
    `env` program from a mangled one, because neither was ever looked at. So
    this probe corrupts one, and requires red.
    """
    print("\n9. check_fixtures reads a program carried in a step's `env` (batch 8B)")
    rs = source_path("browser_runtime_summary_fallback_ts_input.rs")
    src = os.path.join(TESTS, "cases/browser/runtime_summary_fallback_ts_input.toml")

    rc, out = run(os.path.join(HERE, "check_fixtures.py"), rs, src)
    check("CONTROL: the shipped pair is green on fixtures",
          rc == 0 and "FIXTURE CHECK OK" in out, f"rc={rc} {out[:160]}")

    text = open(src).read()
    needle = 'fs.writeFileSync(process.env.KALI_BROWSER_HARNESS_SUMMARY_FILE, \\"not-json\\")'
    check("POISON PRECONDITION: the env program is present to corrupt",
          needle in text, needle[:80])
    poisoned = os.path.join(tmp, "env_poisoned.toml")
    open(poisoned, "w").write(text.replace(
        needle,
        needle.replace('\\"not-json\\"', '\\"not-jsonX\\"'), 1))
    rc, out = run(os.path.join(HERE, "check_fixtures.py"), rs, poisoned)
    check("POISON: one corrupted byte in an `env` program turns the arm RED",
          rc == 1 and "UNMATCHED" in out, f"rc={rc} {out[:200]}")

    # One layer up: with the `env` limb removed the poison must stop being
    # caught, or the limb is not what is catching it.
    import check_fixtures as C
    real = C.toml_program_texts
    try:
        def without_env(path):
            import tomllib as _t
            doc = _t.load(open(path, "rb"))
            out_ = [v for v in (doc.get("source") or {}).values() if isinstance(v, str)]
            for case in doc.get("case") or []:
                for st in (case.get("step") or [case]):
                    if isinstance(st, dict) and isinstance(st.get("body"), str):
                        out_.append(st["body"])
            return out_
        C.toml_program_texts = without_env
        buf_clean, buf_poison = io.StringIO(), io.StringIO()
        with contextlib.redirect_stdout(buf_clean):
            rc_clean = C.main([rs, src])
        with contextlib.redirect_stdout(buf_poison):
            rc_poison = C.main([rs, poisoned])
    finally:
        C.toml_program_texts = real
    # THE CLAIM IS INDISTINGUISHABILITY, NOT REDNESS. A first version asserted
    # only `rc == 1` with the limb removed -- which passes because ALL 23 env
    # programs are then unmatched, i.e. for a reason that has nothing to do with
    # the poison. A limb that would pass whether or not the poison were there
    # measures nothing, which is the exact defect this whole section exists to
    # rule out. So: with the limb removed, the CLEAN pair and the POISONED pair
    # must produce byte-identical output -- the arm genuinely cannot tell them
    # apart -- and with it restored they must differ.
    check("KILL POWER: without the `env` limb, clean and poisoned are "
          "INDISTINGUISHABLE (same rc, byte-identical output)",
          (rc_clean, buf_clean.getvalue()) == (rc_poison, buf_poison.getvalue()),
          f"rc_clean={rc_clean} rc_poison={rc_poison}")
    rc_a, out_a = run(os.path.join(HERE, "check_fixtures.py"), rs, src)
    rc_b, out_b = run(os.path.join(HERE, "check_fixtures.py"), rs, poisoned)
    check("RESTORED: with the limb, clean and poisoned DO differ",
          (rc_a, out_a) != (rc_b, out_b) and rc_a == 0 and rc_b == 1,
          f"rc_clean={rc_a} rc_poison={rc_b}")

    # The other half of batch 8B's check_fixtures work: PROGRAM_HINT gained a
    # `let <ident> =` alternative because `browser_wasm_threads_browser_surface.rs`'s
    # entire program under test is `let value = 1 + 2; value;` and matched none
    # of the older alternatives -- so the arm found NO fixture and returned its
    # vacuity floor, i.e. it checked nothing for that whole target.
    ws_rs = source_path("browser_wasm_threads_browser_surface.rs")
    ws_toml = os.path.join(TESTS, "cases/browser/wasm_threads_browser_surface_explicit_api.toml")
    rc, out = run(os.path.join(HERE, "check_fixtures.py"), ws_rs, ws_toml)
    check("CONTROL: the `let ...` program is now FOUND and matched (not vacuous)",
          rc == 0 and "FIXTURE CHECK OK" in out and "VACUOUS" not in out,
          f"rc={rc} {out[:140]}")

    ws_poison = os.path.join(tmp, "ws_poisoned.toml")
    ws_text = open(ws_toml).read()
    check("POISON PRECONDITION: the `let ...` program is present to corrupt",
          "let value = 1 + 2; value;" in ws_text, ws_text[:120])
    open(ws_poison, "w").write(
        ws_text.replace("let value = 1 + 2; value;", "let value = 1 + 3; value;", 1))
    rc, out = run(os.path.join(HERE, "check_fixtures.py"), ws_rs, ws_poison)
    check("POISON: a corrupted `let ...` program turns the arm RED",
          rc == 1 and "UNMATCHED" in out, f"rc={rc} {out[:200]}")

    # One layer up: with the alternative removed the SAME poison must stop being
    # caught -- and it must stop being caught by going VACUOUS, which is the
    # specific way this defect hid.
    import re as _re
    real_hint = C.PROGRAM_HINT
    try:
        C.PROGRAM_HINT = _re.compile(
            r"console\.log|Kali\.test|await import|function\s|const\s|export\s")
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            rc_old = C.main([ws_rs, ws_poison])
    finally:
        C.PROGRAM_HINT = real_hint
    check("KILL POWER: without the `let ...` alternative the same poison is not "
          "caught -- the arm reports VACUOUS and checks nothing",
          rc_old == 2 and "VACUOUS" in buf.getvalue(),
          f"rc={rc_old} {buf.getvalue()[:140]}")


# EVERY PROBE THIS FILE PROMISES TO RUN, DECLARED SO THE COUNT IS GATED.
#
# 8C's first round fixed the INSTANCE (a `FileNotFoundError` reading a deleted
# source) and not the CLASS it had just named: nine probes called in a row, no
# per-probe guard, and a raise in any of them skips every probe after it AND the
# `FAILURES` block, so the process dies with a traceback that -- from an exit
# code alone -- is indistinguishable from "one arm reported a defect". Worse, a
# raise in the LAST probe would have left `FAILURES` empty and printed nothing
# at all.
#
# Each probe is now run behind its own guard, a crash is recorded as a failure
# rather than ending the run, and the number that actually ran is compared with
# this declaration -- the gate's own output against its own declaration
# (ruling 15 answer 1), so deleting a call from the list below fails here
# instead of quietly shrinking the suite.
PROBES_DECLARED = 9


def main():
    tmp = tempfile.mkdtemp(prefix="inst2-probes-")
    probes = [
        ("argv correspondence", lambda: probe_argv_correspondence(tmp)),
        ("`.replace` arm", lambda: probe_replace_arm(tmp)),
        ("ghost declarations", probe_ghost_declarations),
        ("ref_carries", probe_ref_carries),
        ("verify_pair delegation", lambda: probe_verify_pair_delegation(tmp)),
        ("population banner", probe_population_banner),
        ("supporting arms", lambda: probe_supporting_arms(tmp)),
        ("structure-arm disclosure", lambda: probe_structure_arm_disclosure(tmp)),
        ("env program texts", lambda: probe_env_program_texts(tmp)),
    ]
    ran = 0
    try:
        for label, fn in probes:
            try:
                fn()
                ran += 1
            except BaseException as exc:                 # noqa: BLE001
                # BaseException, not Exception: a probe that calls `sys.exit`
                # raises SystemExit, which is exactly the "a raise skips the
                # FAILURES block" class this guard exists to close, and it is
                # NOT an Exception.
                import traceback
                check(f"probe {label!r} completed without raising", False,
                      f"{type(exc).__name__}: {exc}")
                traceback.print_exc()
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    check(f"all {PROBES_DECLARED} declared probe(s) ran", ran == PROBES_DECLARED,
          f"{ran} completed, PROBES_DECLARED says {PROBES_DECLARED} -- a suite "
          "that quietly runs fewer arms than it claims reports a green that "
          "means less than the reader thinks it does")
    if FAILURES:
        print(f"\nPROBES FAILED -- {len(FAILURES)} arm(s) did not behave as "
              "declared")
        for f in FAILURES:
            print(f"  {f}")
        return 1
    # NO QUANTIFIER OVER "every arm this dispatch added" (round 1, I1). That
    # sentence was a ruling-13 universal about this file's own completeness, and
    # it was false: several arms in the same diff were not probed. What is probed
    # is the list in the docstring, and what is not is named there too.
    print("\nPROBES OK -- the nine sections above each fired on the defect they "
          "exist to catch and stayed silent on their control; see this file's "
          "docstring for what is probed and what is not")
    return 0


if __name__ == "__main__":
    sys.exit(main())
