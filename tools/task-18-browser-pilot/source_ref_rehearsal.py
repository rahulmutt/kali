#!/usr/bin/env python3
"""Task 18 batch 8: rehearse the family deletion against the citation gate.

WHY THIS EXISTS. Task 18's last step deletes every `browser_*.rs` in one commit.
Every `:N` in the migrated case files resolves against those sources, so the
moment they go, the gate's two RESOLVING arms have nothing to resolve against
and only the GATEDNESS arm survives. `citation_sweep.sh`'s SOURCE-DELETED arm
makes such a case file resolve against the historical blob named by its own
`SOURCE REF:` line instead. A green sweep is not evidence that it does: a
removed check is silent by definition, and an arm that reports nothing looks
exactly like an arm that finds nothing (rulings 15 and 18).

So this rehearses the deletion, on a scratch copy, and gates on the result:

  1. EQUIVALENCE. A sample of stems whose `.rs` is in the tree -- including a
     `#[path]` submodule carrier, whose sibling directory U10 deletes with it --
     is swept twice: once as shipped, once with the `.rs` (and the sibling
     directory) removed and only the `SOURCE REF:` declaration left behind. The
     two sweep outputs must be BYTE-IDENTICAL. The declaration is added to the
     case file BEFORE the first run, so the only difference between the runs is
     the presence of the sources -- which is what the deletion commit is.
  2. KILL POWER, IN THE DELETED STATE. Poisons that the gate catches while the
     source is present must still be caught once it is gone. A probe that passes
     in the with-source state and not in the deleted state is the whole defect
     this mechanism exists to prevent, and it is invisible to (1).
  3. THE THREE FAILURE MODES of a `SOURCE REF:` declaration, each demonstrated
     exiting 2 rather than falling through to the gatedness-only path.
  4. POPULATION AGREEMENT. `citation_tiers.py` carries a second copy of the
     driver's population loop, and it is what regenerates `NO_NEEDLE_DECLARED` /
     `PINNED_SPLIT_DECLARED`. The two must build the same population.
  5. THE SELFTEST'S OWN KILL POWER. `--selftest`'s new SOURCE-REF probe is
     checked one layer up, by deleting the line it guards and requiring
     `--selftest` to fail.

Nothing here mutates the real tree: every run happens in a scratch clone under
`mktemp -d`, made with `git clone --shared --no-checkout` (instant, ~200KB, and
it can still read every historical blob through its alternates). Half-cleaning
the real tree with `git checkout` is expensive in this project and is exactly
what the brief forbids.

    python3 tools/task-18-browser-pilot/source_ref_rehearsal.py

Exit 0 if every check passes, 1 otherwise.
"""

import difflib
import os
import re
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
TESTS_REL = "crates/kali_cli/tests"
sys.path.insert(0, HERE)
from submodules import submodule_paths  # noqa: E402

# THE SAMPLE. Named here, but every property this rehearsal relies on is
# ASSERTED below rather than described: `.rs` present, no `//!` retention header
# (the deletion removes a retention header along with its file, so a stem that
# has one cannot have byte-identical output on both sides -- that is a real
# difference, not an artefact to explain away), a `Migrated from` line, and at
# least one `#[path]` carrier among them.
SAMPLE = [
    "object_keys_iteration",                    # the `#[path]` carrier (U10)
    "math_pow_zero_exponent_non_integer_base",  # 2 UNGATED_REDLIST entries
    "math_sqrt_cbrt_harness",                   # a declared bare-needle stem
]

MIGRATED = re.compile(r"^#\s*Migrated from tests/(browser_[A-Za-z0-9_]+\.rs)",
                      re.M)
# A citation with a backticked construct in front of it, in both the qualified
# `(build.rs:N)` and the bare `(:N)` form. Deliberately narrower than
# `batch5_crosscheck.CITE`: this only has to FIND a poisonable site, and every
# poison it produces is validated against the with-source run before its
# deleted-state verdict is trusted.
QUALIFIED_CITE = re.compile(r"(`[^`\n]{3,200}`[^`\n]{0,40}?\()([A-Za-z0-9_]+\.rs):(\d+)(\))")
BARE_CITE = re.compile(r"(`[^`\n]{3,200}`[^`\n]{0,40}?\():(\d+)(\))")

FAILURES = []
NOTE = []


def fail(msg):
    FAILURES.append(msg)
    print(f"  FAIL: {msg}")


def git(*args, cwd=REPO):
    return subprocess.run(("git", "-C", cwd) + args,
                          capture_output=True, text=True)


def head_sha():
    out = git("rev-parse", "HEAD")
    assert out.returncode == 0, out.stderr
    return out.stdout.strip()


def build_scratch():
    """A throwaway checkout that can still read history, populated from the
    WORKING TREE so uncommitted gate edits are what gets rehearsed."""
    root = tempfile.mkdtemp(prefix="source-ref-rehearsal-")
    scratch = os.path.join(root, "repo")
    out = subprocess.run(
        ["git", "clone", "--shared", "--no-checkout", "--quiet", REPO, scratch],
        capture_output=True, text=True)
    if out.returncode:
        sys.exit(f"cannot clone {REPO} into the scratch: {out.stderr.strip()}")
    for rel in (TESTS_REL, "tools/task-18-browser-pilot", "scripts"):
        dst = os.path.join(scratch, rel)
        os.makedirs(os.path.dirname(dst), exist_ok=True)
        shutil.copytree(os.path.join(REPO, rel), dst,
                        ignore=shutil.ignore_patterns("__pycache__"))
    return root, scratch


def sweep(scratch, args=()):
    out = subprocess.run(
        ["bash", os.path.join(scratch, "tools/task-18-browser-pilot/citation_sweep.sh"),
         *args],
        capture_output=True, text=True)
    return out.returncode, out.stdout + out.stderr


def toml_path(scratch, stem):
    return os.path.join(scratch, TESTS_REL, "cases/browser", f"{stem}.toml")


def declare(scratch, stem, sha):
    """Insert `SOURCE REF: <sha>` under the case file's `Migrated from` line."""
    path = toml_path(scratch, stem)
    text = open(path).read()
    m = MIGRATED.search(text)
    assert m, path
    at = text.index("\n", m.start()) + 1
    open(path, "w").write(text[:at] + f"#   SOURCE REF: {sha}\n" + text[at:])
    return m.group(1)


def delete_source(scratch, src):
    """Remove `src` and, if it is a `#[path]` carrier, its sibling files --
    U10's unit, and what the family-deletion commit removes."""
    live = os.path.join(REPO, TESTS_REL, src)
    removed = []
    for sub in submodule_paths(live):
        rel = os.path.relpath(str(sub), os.path.join(REPO, TESTS_REL))
        target = os.path.join(scratch, TESTS_REL, rel)
        if os.path.exists(target):
            os.unlink(target)
            removed.append(rel)
    os.unlink(os.path.join(scratch, TESTS_REL, src))
    for rel in removed:                       # leave no empty sibling directory
        d = os.path.dirname(os.path.join(scratch, TESTS_REL, rel))
        if os.path.isdir(d) and not os.listdir(d):
            os.rmdir(d)
    return removed


# ---------------------------------------------------------------------------
# 0. Preconditions on the sample, derived rather than assumed.
# ---------------------------------------------------------------------------
def check_sample():
    carriers = 0
    for stem in SAMPLE:
        rs = os.path.join(REPO, TESTS_REL, f"browser_{stem}.rs")
        toml = os.path.join(REPO, TESTS_REL, "cases/browser", f"{stem}.toml")
        if not os.path.exists(rs) or not os.path.exists(toml):
            fail(f"sample stem {stem}: {rs} or {toml} is missing")
            continue
        text = open(rs).read()
        if any(l.startswith("//!") for l in text.split("\n")):
            fail(f"sample stem {stem}: its `.rs` carries a `//!` retention "
                 "header, which the deletion removes along with the file -- the "
                 "two sides cannot be byte-identical, so this stem cannot "
                 "measure equivalence")
        if not MIGRATED.search(open(toml).read()):
            fail(f"sample stem {stem}: no `Migrated from tests/<file>.rs` line, "
                 "so the sweep cannot name its source once the file is gone")
        if "SOURCE REF:" in open(toml).read():
            fail(f"sample stem {stem}: already declares a SOURCE REF -- this "
                 "rehearsal adds one and would double it")
        subs = submodule_paths(rs)
        carriers += bool(subs)
        # The declared ref is HEAD, so the working tree copy of the source (and
        # of its submodules) has to BE HEAD's, or the two sides differ for a
        # reason that has nothing to do with the mechanism.
        paths = [f"{TESTS_REL}/browser_{stem}.rs"] + [
            f"{TESTS_REL}/{os.path.relpath(str(p), os.path.join(REPO, TESTS_REL))}"
            for p in subs]
        dirty = git("diff", "--quiet", "HEAD", "--", *paths)
        if dirty.returncode:
            fail(f"sample stem {stem}: {paths} differ from HEAD. The rehearsal "
                 "declares HEAD as the SOURCE REF, so commit the sources first.")
        print(f"  sample {stem}: no retention header, "
              f"{len(subs)} submodule file(s), matches HEAD")
    if not carriers:
        fail("the sample contains no `#[path]` submodule carrier, so it does "
             "not rehearse the U10 shape at all -- which is the shape §2.4 says "
             "arrives next")


# ---------------------------------------------------------------------------
# 1. The equivalence rehearsal.
# ---------------------------------------------------------------------------
def rehearse(scratch, sha):
    srcs = [declare(scratch, stem, sha) for stem in SAMPLE]
    rc_with, out_with = sweep(scratch)
    if rc_with != 0:
        fail(f"the WITH-SOURCE baseline did not exit 0 (rc={rc_with}); the "
             f"rehearsal cannot mean anything against a red baseline:\n"
             f"{out_with[-2000:]}")
        return None, None, srcs
    removed = []
    for src in srcs:
        removed += delete_source(scratch, src)
    print(f"  deleted {len(srcs)} source(s) and {len(removed)} submodule "
          f"file(s): {', '.join(srcs + removed)}")
    rc_del, out_del = sweep(scratch)
    if rc_del != 0:
        fail(f"the DELETED-state sweep did not exit 0 (rc={rc_del}):\n"
             f"{out_del[-3000:]}")
    diff = list(difflib.unified_diff(
        out_with.split("\n"), out_del.split("\n"),
        "with-source", "source-deleted", lineterm="", n=1))
    print("  --- per-stem lines, both sides ---")
    for stem in SAMPLE:
        for label, out in (("with-source  ", out_with),
                           ("source-deleted", out_del)):
            line = next((l for l in out.split("\n")
                         if l.startswith(f"{stem}:")), "<absent>")
            print(f"    {label}  {line}")
    if diff:
        fail("the two sweeps differ -- the deletion changed what the gate reads:"
             + "\n" + "\n".join(diff[:60]))
    else:
        print("  --- diff(with-source, source-deleted): EMPTY ---")
    return out_with, out_del, srcs


# ---------------------------------------------------------------------------
# 2. Kill power, both sides.
# ---------------------------------------------------------------------------
def poison_drift(text, qualified):
    """Every +1 drift of a citation that has a construct in front of it."""
    pat = QUALIFIED_CITE if qualified else BARE_CITE
    for m in pat.finditer(text):
        n = int(m.group(3) if qualified else m.group(2))
        if qualified:
            new = f"{m.group(1)}{m.group(2)}:{n + 1}{m.group(4)}"
        else:
            new = f"{m.group(1)}:{n + 1}{m.group(3)}"
        yield (f"drift {'qualified' if qualified else 'bare'} citation "
               f"{m.group(0).strip()} by +1"), text[:m.start()] + new + text[m.end():]


def poison_unbacktick(text):
    """Strip the backticks off the construct a citation rests on: the citation
    stays WRITTEN and stops being MATCHED, which is `_gated_arm`'s whole
    subject."""
    for m in BARE_CITE.finditer(text):
        opened = m.group(1)
        stripped = opened.replace("`", "", 2)
        new = stripped + f":{m.group(2)}" + m.group(3)
        yield (f"un-backtick the construct in front of {m.group(0).strip()}",
               text[:m.start()] + new + text[m.end():])


def poison_strand_redlist(text, keys):
    """Remove the citations an UNGATED_REDLIST entry fires on, so the entry is
    left pointing at nothing."""
    line = next((l for l in text.split("\n")
                 if all(k in l for k in keys)), None)
    if line is None:
        return
    scrubbed = line
    for k in keys:
        scrubbed = scrubbed.replace(k, " elsewhere")
    yield (f"strand the UNGATED_REDLIST entries {', '.join(keys)}",
           text.replace(line, scrubbed))


def kill_power(scratch, sha, srcs):
    """Each poison is applied twice: to the tree as shipped (which is what
    proves the poison HAS kill power) and to the deleted state (which is what
    this task is about). The first candidate whose with-source run fails is the
    one reported; a poison that fails to fail with the source present is not
    evidence about anything and is skipped."""
    import batch5_crosscheck as X

    carrier = SAMPLE[0]
    redlist_stem = "math_pow_zero_exponent_non_integer_base"
    keys = sorted({k[2] for k in X.UNGATED_REDLIST if k[0] == redlist_stem})
    if not keys:
        fail(f"no UNGATED_REDLIST entry for {redlist_stem}; the stranding probe "
             "has nothing to strand")

    plans = [
        (carrier, lambda t: poison_drift(t, qualified=True)),
        ("math_sqrt_cbrt_harness", lambda t: poison_drift(t, qualified=False)),
        ("math_sqrt_cbrt_harness", poison_unbacktick),
        (redlist_stem, lambda t: poison_strand_redlist(t, keys)),
    ]
    for stem, gen in plans:
        path = toml_path(scratch, stem)
        pristine = open(path).read()
        reported = False
        for label, poisoned in gen(pristine):
            # The deleted state is what the scratch is already in; restore the
            # source for the with-source half, poison, measure, delete again.
            open(path, "w").write(poisoned)
            rc_del, out_del = sweep(scratch)
            restore_sources(scratch, srcs)
            rc_with, out_with = sweep(scratch)
            for src in srcs:
                delete_source(scratch, src)
            open(path, "w").write(pristine)
            if rc_with == 0:
                continue        # no kill power with the source present: skip
            reported = True
            verdict = "CAUGHT" if rc_del else "SILENT"
            print(f"  {stem}: {label}\n"
                  f"      with-source    rc={rc_with}  CAUGHT\n"
                  f"      source-deleted rc={rc_del}  {verdict}")
            if not rc_del:
                fail(f"{stem}: {label} is caught with the source present and "
                     "SILENT once it is deleted -- the resolving arms are not "
                     "firing through the SOURCE REF blob")
            break
        if not reported:
            fail(f"{stem}: no candidate for this poison failed even WITH the "
                 "source present, so the probe has no kill power to compare")


def restore_sources(scratch, srcs):
    for src in srcs:
        live = os.path.join(REPO, TESTS_REL, src)
        shutil.copy2(live, os.path.join(scratch, TESTS_REL, src))
        for sub in submodule_paths(live):
            rel = os.path.relpath(str(sub), os.path.join(REPO, TESTS_REL))
            dst = os.path.join(scratch, TESTS_REL, rel)
            os.makedirs(os.path.dirname(dst), exist_ok=True)
            shutil.copy2(str(sub), dst)


# ---------------------------------------------------------------------------
# 3. The three failure modes.
# ---------------------------------------------------------------------------
def failure_modes(scratch, sha):
    stem = SAMPLE[0]
    path = toml_path(scratch, stem)
    pristine = open(path).read()
    # A commit that exists and does NOT contain the cited source: the parent of
    # the commit that ADDED it. Derived, because "the deletion commit's parent,
    # not the deletion commit" is the mistake this mode exists to catch and a
    # hand-picked sha would not demonstrate it.
    src = MIGRATED.search(pristine).group(1)
    added = git("log", "--diff-filter=A", "-1", "--format=%H", "--",
                f"{TESTS_REL}/{src}").stdout.strip()
    before_add = git("rev-parse", f"{added}^").stdout.strip() if added else ""
    cases = [
        ("no declaration at all",
         pristine.replace(f"#   SOURCE REF: {sha}\n", ""),
         "declares no `SOURCE REF:`"),
        ("a branch name instead of a full sha",
         pristine.replace(sha, "main"),
         "is not a full 40-char sha"),
        ("an abbreviated sha",
         pristine.replace(sha, sha[:10]),
         "is not a full 40-char sha"),
        ("a well-formed sha that is not in the repository",
         pristine.replace(sha, "deadbeef" * 5),
         "fetch-depth: 0"),
        ("two declarations",
         pristine.replace(f"#   SOURCE REF: {sha}\n",
                          f"#   SOURCE REF: {sha}\n#   SOURCE REF: {sha}\n"),
         "declares 2 `SOURCE REF:` lines"),
        ("a declaration below the header rather than in it",
         pristine.replace(f"#   SOURCE REF: {sha}\n", "")
         + f"\n#   SOURCE REF: {sha}\n",
         "declares no `SOURCE REF:`"),
    ]
    if before_add and len(before_add) == 40:
        cases.append(("a commit that predates the source's own addition "
                      f"({before_add[:10]}) -- the shape of naming the deletion "
                      "commit instead of its parent",
                      pristine.replace(sha, before_add),
                      "does not contain"))
    else:
        NOTE.append(f"could not derive a commit predating {src}'s addition "
                    "(it appears in the root commit), so failure mode 3 is "
                    "probed only by the unreachable-ref case")
    try:
        for label, text, needle in cases:
            open(path, "w").write(text)
            rc, out = sweep(scratch)
            hit = needle in out
            print(f"  {label}: rc={rc}, message names "
                  f"{'the remedy' if hit else 'SOMETHING ELSE'}")
            if rc != 2 or not hit:
                fail(f"failure mode `{label}`: expected exit 2 with a message "
                     f"containing {needle!r}, got rc={rc}:\n{out[-1500:]}")
    finally:
        open(path, "w").write(pristine)


# ---------------------------------------------------------------------------
# 4/5. The two one-layer-up checks.
# ---------------------------------------------------------------------------
def population_agreement(scratch):
    rc, shell = sweep(scratch, ("--print-specs",))
    tiers = subprocess.run(
        [sys.executable,
         os.path.join(scratch, "tools/task-18-browser-pilot/citation_tiers.py"),
         "--specs"], capture_output=True, text=True)
    if rc or tiers.returncode:
        fail(f"population listing failed (sweep rc={rc}, tiers "
             f"rc={tiers.returncode}): {tiers.stderr[-800:]}")
        return
    a = sorted(l for l in shell.split("\n") if l.strip())
    b = sorted(l for l in tiers.stdout.split("\n") if l.strip())
    print(f"  citation_sweep.sh: {len(a)} spec(s); citation_tiers.py: "
          f"{len(b)} spec(s)")
    if a != b:
        fail("the two population loops disagree:\n" + "\n".join(
            difflib.unified_diff(a, b, "citation_sweep.sh", "citation_tiers.py",
                                 lineterm="")))


def selftest_kill_power(scratch):
    """`--selftest`'s SOURCE-REF probe, checked one layer up: delete the line it
    guards and the selftest must go red."""
    path = os.path.join(scratch, "tools/task-18-browser-pilot/batch5_crosscheck.py")
    pristine = open(path).read()
    guarded = "        bases.append(rs_path)\n"
    if pristine.count(guarded) != 1:
        fail("cannot locate the single `bases.append(rs_path)` line the SOURCE "
             "REF probe guards -- the mutation cannot be applied")
        return
    try:
        for label, text, want_rc in (
                ("shipped", pristine, 0),
                ("with `bases.append(rs_path)` removed",
                 pristine.replace(guarded, "        pass\n"), 1)):
            open(path, "w").write(text)
            out = subprocess.run([sys.executable, path, "--selftest"],
                                 capture_output=True, text=True)
            print(f"  --selftest {label}: rc={out.returncode}")
            if out.returncode != want_rc:
                fail(f"--selftest {label}: rc={out.returncode}, expected "
                     f"{want_rc}. A probe that stays green when the line it "
                     "guards is deleted is not wired to anything.\n"
                     f"{out.stdout[-1200:]}")
    finally:
        open(path, "w").write(pristine)


def main():
    sha = head_sha()
    print(f"REPO={REPO}\nSOURCE REF used by the rehearsal = {sha}\n")
    print("0. sample preconditions")
    check_sample()
    root, scratch = build_scratch()
    try:
        print("\n4. population agreement (citation_sweep.sh vs citation_tiers.py)")
        population_agreement(scratch)
        print("\n5. --selftest kill power on the new base")
        selftest_kill_power(scratch)
        print("\n1. equivalence rehearsal")
        _, _, srcs = rehearse(scratch, sha)
        print("\n2. kill power in the deleted state")
        kill_power(scratch, sha, srcs)
        print("\n3. failure modes of a SOURCE REF declaration")
        failure_modes(scratch, sha)
    finally:
        shutil.rmtree(root, ignore_errors=True)
    for n in NOTE:
        print(f"\nNOTE: {n}")
    if FAILURES:
        print(f"\nREHEARSAL FAILED -- {len(FAILURES)} problem(s)")
        for f in FAILURES:
            print(f"  {f}")
        return 1
    print("\nREHEARSAL OK — with the sources deleted and only `SOURCE REF:` left, "
          "the sweep reads the same citations, catches the same poisons, and "
          "refuses to run at all on a declaration it cannot resolve")
    return 0


if __name__ == "__main__":
    sys.exit(main())
