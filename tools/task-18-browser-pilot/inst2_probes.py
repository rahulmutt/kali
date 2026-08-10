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
    rs = os.path.join(TESTS, "browser_object_from_entries_harness.rs")
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

    sourced = "promise_all_bundle"          # has browser_<stem>.rs
    split = "reflect_own_keys_explicit_api"  # U2 split of a U4-TRIMMED carrier
    no_trim_split = "non_literal_iterator_sources_explicit_api"   # U2 split, no trim

    def banner(stem):
        _rc, out = run(path, stem)
        return out

    control = banner(sourced)
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
    import re as _re
    for stem, must_work in ((no_trim_split, True), (split, False)):
        text = banner(stem)
        m = _re.search(r"`([a-z0-9_]+=crates/kali_cli/tests/[^`]+)`", text)
        if must_work:
            check(f"HINT IS FOLLOWABLE: {stem} emits an override and it is offered",
                  m is not None, text[:200])
            if m:
                _rc, out = run(path, m.group(1))
                check(f"HINT IS FOLLOWABLE: obeying `{m.group(1)[:52]}...` "
                      "produces no past-end failure",
                      "past end of the source" not in out, out[-200:])
        else:
            # The trimmed carrier: the same form must be WARNED AGAINST, and the
            # warning must be true -- so obey it anyway and require it to fail.
            check(f"HINT WARNS: {stem} says that override would resolve against "
                  "the TRIMMED file",
                  "would resolve against the TRIMMED file" in text, text[:200])
            if m:
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


def main():
    tmp = tempfile.mkdtemp(prefix="inst2-probes-")
    try:
        probe_argv_correspondence(tmp)
        probe_replace_arm(tmp)
        probe_ghost_declarations()
        probe_ref_carries()
        probe_verify_pair_delegation(tmp)
        probe_population_banner()
        probe_supporting_arms(tmp)
        probe_structure_arm_disclosure(tmp)
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
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
    print("\nPROBES OK -- the eight sections above each fired on the defect they "
          "exist to catch and stayed silent on their control; see this file's "
          "docstring for what is probed and what is not")
    return 0


if __name__ == "__main__":
    sys.exit(main())
