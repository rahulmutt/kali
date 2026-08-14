#!/usr/bin/env python3
"""Screen non-browser migration candidates against every known blocking shape.

WHY THIS IS A GATED SCRIPT AND NOT A GREP (U12). Task 19's pilot ran its first
screen as an ad-hoc regex sweep, reported `60 clean / 632 tests`, and that figure
was materially wrong in two independent ways:

  1. It scored FOUR targets clean that carry `//!` headers SAYING THEY COULD NOT
     BE MIGRATED -- `soundness_abort`, `soundness_param_truncation`,
     `soundness_url`, `inprocess`. A screen that scores a self-documented
     retention clean is validated against nothing.
  2. Its denominator omitted every `#[test]` behind a `#[path]` submodule. Eight
     targets carry 2279 such tests; the reported 767 counted none of them
     (all eight declare zero top-level tests, so a top-level `grep -c '#[test]'`
     returns 0 and silently drops the file's entire contents -- U10's failure
     mode, in the screen rather than in the audit).

So the ground truth is wired in below and `--selftest` FAILS (exit 1) if the
screen scores any of it clean. This is the project's own "obey the artifact,
check the result against a known positive, never trust a zero you did not try to
make non-zero" rule applied to the INSTRUMENT rather than to the artifact.

  KNOWN_BLOCKED is load-bearing. Every newly adjudicated retention must be added
  to it, or the selftest silently weakens as the corpus grows (ruling 10's
  lesson, borrowed).

VERDICTS. Three, not two -- a binary clean/blocked screen is what produced the
first version's overconfidence:

  BLOCKED     a shape with no expressible form; §5.11 retention.
  ADJUDICATE  a shape that is sometimes migratable and needs a per-site call.
              Reported separately and never counted as clean.
  CLEAN       no known blocking or adjudicable shape.

THE RETENTION CROSS-CHECK (added in Task 19 batch 2). `citation_sweep.sh`'s
whole-file-retention arm adopts, for a prefixed family, every `<prefix><stem>.rs`
that carries a `//!` header and has no `cases/<family>/<stem>.toml`. Nothing in
the tree distinguishes a U3 retention header from an ordinary module doc, so an
UNMIGRATED target with a module doc is adopted as a retention and then passes
vacuously -- measured at one instance, `runtime_monomorphize.rs`, before this
batch migrated it. The screen already knows which targets are migratable, so the
two populations can be compared: every adopted retention must be a target this
screen calls BLOCKED. `--retention-crosscheck` is that comparison, and it runs
inside `--selftest` so it is re-run by the gate lane rather than by whoever
remembers. It carries its own known positive: a green cross-check that has never
been made red is not evidence.

Usage:
  screen_candidates.py                       # full report
  screen_candidates.py --selftest            # gate: ground truth + cross-check
  screen_candidates.py --list-clean          # the work list, one stem per line
  screen_candidates.py --retention-crosscheck  # that arm alone
"""

from __future__ import annotations

import os
import re
import sys

TESTS = os.path.join(
    os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))),
    "crates/kali_cli/tests",
)

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import t19_sources as T19S  # noqa: E402

# --- ground truth -----------------------------------------------------------
# Targets whose own `//!` header states they cannot be migrated. The screen must
# never score one of these CLEAN. Sourced by reading the headers, not inferred.
KNOWN_BLOCKED = {
    "soundness_abort",              # "could not migrate to a case file"
    "soundness_console_multiarg",   # "cannot express", "hand-written per spec 5.11"
    "soundness_param_truncation",   # "the ONE test this family could not migrate"
    "soundness_url",                # "could not migrate to a case file"
    "inprocess",                    # "cannot be driven through the `kali` binary"
}

# Targets already migrated by the Task 19 pilot: the screen must score these
# CLEAN, or it is over-blocking. A screen validated only against known-blocked
# ground truth can trivially pass by blocking everything.
KNOWN_CLEAN = {
    "runtime_multi_declarator",
    "nullish_assign_reject",
    "growable_array_fail_closed",
    "compound_assignment_wrapped_local_binding",
    # `bitwise_operators_runtime` is deliberately NOT here: it carries the
    # combined stdout+stderr shape and screens as ADJUDICATE. It was migrated
    # after adjudicating that site (rule 11 + rule 2's presence/absence
    # asymmetry), which is exactly what ADJUDICATE is supposed to prompt.
    #
    # `template_literal_interpolation_runtime` is NOT here for the same reason
    # and it is the second instance: Task 19 batch 4 widened S26 to match its
    # `(stdout.clone() + &stderr)` spelling, which moved it CLEAN -> ADJUDICATE,
    # and then migrated it in the same batch after resolving the site against the
    # real binary (the needle is on stderr, on the one cell that carries it, and
    # the claim is a PRESENCE claim so rule 2's asymmetry does not bite). A
    # target that screens ADJUDICATE and has been adjudicated is migratable; the
    # verdict is a prompt, not a refusal, and this list is only for targets that
    # must screen CLEAN.
    #
    # A LIMITATION OF THIS LIST, STATED SO IT IS NOT MISTAKEN FOR COVERAGE: every
    # target that carries an S26 shape is deliberately absent from it, so
    # `KNOWN_CLEAN` structurally CANNOT catch an over-wide S26. It bounds
    # over-blocking by the BLOCKING shapes only. What bounds an S26 widening is
    # the measurement recorded above the pattern -- running the screen with and
    # without each alternative and diffing the verdicts -- and the fact that the
    # only targets it moves are ones already migrated after adjudicating exactly
    # that shape.
}

RETENTION_PHRASES = (
    "could not migrate",
    "cannot express",
    "hand-written per spec",
    "cannot be driven through",
    "cannot be black-box",
    "this family could not",
)

# --- shapes -----------------------------------------------------------------
# Each entry: (id, regex, verdict, why)
BLOCKING = [
    ("S1_starts_with",   r"\.starts_with\(",            "position-anchored claim; rule 1 forbids downgrading to contains"),
    ("S2_lines",         r"\.lines\(\)",                "line-structural claim with no key"),
    ("S3_ends_with",     r"\.ends_with\(",              "position-anchored claim; rule 1"),
    ("S4_assert_ne",     r"assert_ne!",                 "inequality claim; no key"),
    ("S5_iter_quant",    r"\.iter\(\)\.(?:all|any)\(",  "quantifier over a collection; no key"),
    ("S6_tree_fixture",  r"tests/fixtures|fn fixture\(","reads a checked-in tree path the trial tempdir cannot reach"),
    ("S7_runtime_hash",  r"Sha256|sha2::|DefaultHasher|blake3|md5",
                                                        "hash over a runtime value; no hash primitive"),
    ("S8_byte_claim",    r"assert(?:_eq)?!\s*\(\s*(?:&\s*)?output\.(?:stdout|stderr)\s*[,)]|output\.(?:stdout|stderr)\s*==|fs::read\(",
                                                        "claim on RAW BYTES; from_utf8_lossy destroys non-UTF-8 before any assertion"),
    ("S21_env_remove",   r"\.env_remove\(",             "SILENT-GREEN: Step::env is a map with no unset; the var is inherited instead"),
    ("S22_ext_oracle",   r'Command::new\(\s*"(?!.*kali)',"drives a non-kali binary as a differential oracle"),
    ("S23_runtime_argv", r"TcpListener|local_addr\(\)", "in-process server / argv derived from a runtime value"),
    ("S24_fs_exists",    r"\.exists\(\)",               "non-JSON filesystem assertion; file_json only reads JSON"),
    ("S25_compile_env",  r"env!\(",                     "compile-time env baked into the expectation"),
]

# S26 WAS WIDENED BY TASK 19 BATCH 4, AND THE POPULATIONS WERE DERIVED BY A SCAN
# INDEPENDENT OF THE PATTERN BEING ADDED.
#
# The original pattern matched two spellings of "build one haystack out of both
# streams". It missed the third, `(stdout.clone() + &stderr)`, so
# `template_literal_interpolation_runtime` screened CLEAN while carrying the
# adjudicable shape; batch 3 measured that and left the decision because widening
# a blocking predicate moves every later batch's scope.
#
# HOW THE POPULATIONS BELOW WERE OBTAINED, and why the first attempt was worth
# discarding. Batch 4's first version grepped the corpus with THE VERY
# ALTERNATIVES IT WAS ADDING and reported "three spellings match nothing today" --
# self-confirming, which is ruling 13's exact target, and false: its `format!` row
# read 0 and the true answer is 3. The rows below come instead from a STRUCTURAL
# scan that knows nothing about this pattern -- find every `format!` call by
# balanced parens (string-aware, so a `)` inside a fixture cannot close it) and
# report the ones whose argument text mentions both streams:
#
#   cd /workspace && python3 - <<'EOF'
#   import re, glob, os, sys
#   sys.path.insert(0, "tools/task-18-browser-pilot")
#   from lexer import find_string_literals
#   ... balanced-paren scan over crates/kali_cli/tests/**/*.rs ...
#   EOF
#   -> runtime_string_equality.rs      "{}{}", String::from_utf8_lossy(&out.stdout), ...
#      soundness_console_multiarg.rs   "{stdout}{stderr}"
#      soundness_param_truncation.rs   "{stdout}{stderr}"
#      3 file(s)
#
# The one the first version's line-oriented grep could not see is
# `runtime_string_equality.rs`: its `format!` spans several lines and its
# arguments contain `::`, which that grep's argument class excluded.
#
# THE FIVE ALTERNATIVES, AND WHAT EACH MATCHES TODAY:
#
#   push_str(&String::from_utf8_lossy(&output.stderr)) / combined.push_str
#                                        2  bitwise_operators_runtime, imperative_core_runtime
#   stdout(.clone())? + &stderr          1  template_literal_interpolation_runtime
#   stderr(.clone())? + &stdout          0
#   format!(.., stdout, .., stderr)      3  runtime_string_equality, soundness_console_multiarg,
#                                           soundness_param_truncation
#   [stdout, stderr].concat()            0
#   stderr.contains(X) || stdout.contains(X)
#                                        2  promise_any_sequencing, set_iteration_runtime
#
# WHAT MOVED, measured by running the screen with and without each alternative
# rather than by reading the regex:
#
#   + the concat spelling      1 target : CLEAN -> ADJUDICATE (template_literal_…)
#   + the format! spelling     0 targets (all three are already BLOCKED by S27 or S21)
#   + the disjunction          2 targets: CLEAN -> ADJUDICATE, and BOTH ARE ALREADY
#                              MIGRATED (promise_any_sequencing by an earlier batch,
#                              set_iteration_runtime by batch 4) -- so no unmigrated
#                              target moves and no later batch's work list changes.
#
# The disjunction is in the pattern because both of those targets resolved it the
# SAME way independently -- pin the stream the binary actually uses, disclose the
# other disjunct (rule 11) -- which is precisely the prompt ADJUDICATE exists to
# give. It was outside S26 before, so both batches had to notice it by hand.
#
# The two alternatives that match nothing today are kept deliberately: a blocking
# predicate that only recognises what the corpus already contains has to be
# widened one spelling at a time by whoever writes the next batch, which is the
# failure the raw-string recogniser class cost this project three batches to
# learn. That claim is about the FUTURE and nothing here confirms it; what is
# measured is the table above.
ADJUDICABLE = [
    ("S26_combined_streams",
     r"push_str\(&String::from_utf8_lossy\(&output\.stderr\)"
     r"|combined\.push_str"
     r"|(?:stdout|out)[A-Za-z0-9_]*(?:\.clone\(\))?\s*\+\s*&\s*[A-Za-z0-9_]*(?:stderr|err)[A-Za-z0-9_]*"
     r"|(?:stderr|err)[A-Za-z0-9_]*(?:\.clone\(\))?\s*\+\s*&\s*[A-Za-z0-9_]*(?:stdout|out)[A-Za-z0-9_]*"
     r"|format!\s*\(\s*(?:b|c)?r?#*\"[^\"]*\"\s*,[^;]{0,400}?\bstdout\b[^;]{0,400}?\bstderr\b"
     r"|format!\s*\(\s*(?:b|c)?r?#*\"[^\"]*\{[^}]*\bstdout\b[^}]*\}[^\"]*\{[^}]*\bstderr\b"
     r"|\[\s*&?[A-Za-z0-9_.]*stdout[^\]]*stderr[^\]]*\]\s*\.concat\(\)"
     r"|\bstderr\b[A-Za-z0-9_.()]*\.contains\([^)]*\)\s*\|\|\s*[A-Za-z0-9_.()]*\bstdout\b[A-Za-z0-9_.()]*\.contains\("
     r"|\bstdout\b[A-Za-z0-9_.()]*\.contains\([^)]*\)\s*\|\|\s*[A-Za-z0-9_.()]*\bstderr\b[A-Za-z0-9_.()]*\.contains\(",
     "asserts against stdout+stderr as ONE surface -- concatenated, formatted "
     "together, or accepted on either by a disjunction; resolvable per rule 11 "
     "for a presence claim, but an absence claim may not be narrowed (rule 2)"),
]

TEST_FN = re.compile(r"#\[test\]")
PATH_MOD = re.compile(r'#\[path\s*=\s*"([^"]+)"\]')


def unit_files(stem: str) -> list[str]:
    """Every file whose `#[test]` fns belong to `stem`: the top-level file plus
    every `#[path]` submodule it declares. U10's rule, applied at screen time --
    a top-level count alone silently returns 0 for the eight carriers."""
    top = os.path.join(TESTS, stem + ".rs")
    files = [top]
    try:
        text = open(top, encoding="utf-8", errors="replace").read()
    except OSError:
        return files
    for rel in PATH_MOD.findall(text):
        cand = os.path.join(TESTS, rel)
        if os.path.isfile(cand):
            files.append(cand)
    return files


def header_says_retained(text: str) -> bool:
    header = []
    for line in text.splitlines():
        if line.startswith("//!"):
            header.append(line.lower())
        elif header:
            break
    blob = " ".join(header)
    return any(p in blob for p in RETENTION_PHRASES)


def screen_one(stem: str) -> dict:
    files = unit_files(stem)
    blobs = []
    for f in files:
        try:
            blobs.append(open(f, encoding="utf-8", errors="replace").read())
        except OSError:
            continue
    return _verdict(stem, blobs, len(files))


def screen_one_anywhere(stem: str) -> dict:
    """`screen_one`, but readable after Task 19's deletion.

    THE SELFTEST'S ARMS USE THIS; `main`'s census does not, and the split is
    deliberate. `KNOWN_CLEAN` bounds this screen's OVER-BLOCKING against four
    targets known to be migratable -- and all four were deleted by Task 19, so
    a tree-only control simply stopped running (it reported "in KNOWN_CLEAN but
    not in the corpus", which is a failure, but the shape one round of "just
    drop them from the list" would have turned into a silent hole). Reading them
    at the pinned deletion ref keeps the control running against exactly the
    same four files, forever.

    The PRODUCTION corpus stays tree-only: "what is left to migrate" is a
    question about the working tree, and answering it from history would inflate
    every count Task 20 reports.
    """
    path = os.path.join(TESTS, stem + ".rs")
    if os.path.exists(path):
        return screen_one(stem)
    return _verdict(stem, [T19S.source_text(stem, quiet=True)], 1)


def _verdict(stem: str, blobs: list[str], n_files: int) -> dict:
    tests = sum(len(TEST_FN.findall(t)) for t in blobs)
    blob = "\n".join(blobs)

    blocked = [(sid, why) for sid, rx, why in BLOCKING if re.search(rx, blob)]
    adjud = [(sid, why) for sid, rx, why in ADJUDICABLE if re.search(rx, blob)]
    if header_says_retained(blobs[0] if blobs else ""):
        blocked.insert(0, ("S27_self_documented",
                           "the file's own `//!` header states it could not be migrated"))

    verdict = "BLOCKED" if blocked else ("ADJUDICATE" if adjud else "CLEAN")
    return {"stem": stem, "tests": tests, "files": n_files,
            "blocked": blocked, "adjudicate": adjud, "verdict": verdict}


def all_stems() -> list[str]:
    out = []
    for name in sorted(os.listdir(TESTS)):
        if not name.endswith(".rs"):
            continue
        stem = name[:-3]
        if stem.startswith("browser_") or stem == "cases":
            continue
        out.append(stem)
    return out


CASES = os.path.join(os.path.dirname(TESTS), "tests", "cases")


def _family_prefixes() -> dict:
    """Each `cases/<family>/`'s source prefix, DERIVED by `families.py`.

    Imported rather than tabulated for the reason `families.py` exists: a table
    mapping every family to `<family>_` is wrong today (`misc/`'s sources carry
    no prefix at all), and a second table would be a second thing to keep in
    step with the first.
    """
    sys.path.insert(0, os.path.join(
        os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))),
        "tools", "task-18-browser-pilot"))
    import families  # noqa: E402
    return {f: families.prefix(f) for f in families.families()}


def retention_adoptions(pretend_missing=frozenset(), *,
                        include_deleted: bool = False) -> list:
    """What `citation_sweep.sh`'s whole-file-retention arm would adopt.

    Reproduces that arm's own predicate -- for a family with a NON-EMPTY prefix,
    a `<prefix><stem>.rs` in the tree with a leading `//!` and no
    `cases/<family>/<stem>.toml`. The empty-prefix families are excluded here for
    the same reason the sweep skips them loudly: with no prefix to filter on, the
    arm adopts every unprefixed `.rs` in the directory, `cases.rs` included.

    `pretend_missing` is the known-positive hook: a set of `(family, stem)` pairs
    whose case file is treated as absent, so the probe can manufacture exactly
    the condition this check exists to catch without touching the tree.

    SCOPE LIMIT, STATED RATHER THAN LEFT TO BE DISCOVERED: this covers only
    families with a NON-EMPTY prefix, so `misc/` -- where most of Task 19 batch
    2's pairs live -- is outside it. That is not a hole under the sweep as it
    stands, because `citation_sweep.sh`'s own retention arm skips an
    empty-prefix family too (it would otherwise adopt every unprefixed `.rs` in
    `tests/`, `cases.rs` included), so there is no adoption there to cross-check.
    It BECOMES a hole the day that arm learns to run on an empty-prefix family,
    and this comment is what should be read then.
    """
    out = []
    for family, prefix in sorted(_family_prefixes().items()):
        if not prefix or family == "browser":
            continue
        for name in _corpus_names(include_deleted):
            if not name.endswith(".rs") or not name.startswith(prefix):
                continue
            stem = name[len(prefix):-3]
            if (family, stem) not in pretend_missing and \
                    os.path.isfile(os.path.join(CASES, family, stem + ".toml")):
                continue
            try:
                text = T19S.source_text(name[:-3], quiet=True)
            except Exception:                       # noqa: BLE001
                continue
            if not text.startswith("//!"):
                continue
            out.append((family, name, name[:-3]))
    return out


def _corpus_names(include_deleted: bool) -> list[str]:
    """The `.rs` names this arm scans: the tree, and optionally what Task 19
    deleted (derived from the pinned ref, never listed).

    `include_deleted` is FALSE for the production arm -- it reproduces
    `citation_sweep.sh`'s tree-scanning predicate and must keep scanning the
    same set that arm does. It is TRUE only for `_crosscheck_probe`, whose known
    positive has to be a migrated, `//!`-carrying, non-BLOCKED, prefixed target,
    and where every such target on disk is one Task 19 deleted. Without this the
    probe reported "no known positive available" -- a control with nothing to
    seed it is not a control, and it is exactly the state the deletion put it
    in.
    """
    names = sorted(os.listdir(TESTS))
    if include_deleted:
        names = sorted(set(names) | {s + ".rs" for s in T19S.deleted_stems()})
    return names


def retention_crosscheck(pretend_missing=frozenset(), *, quiet=False,
                         include_deleted: bool = False) -> list:
    """Adopted retentions that this screen says are migratable. Empty == good."""
    stems = all_stems()
    if include_deleted:
        stems = sorted(set(stems) | set(T19S.deleted_stems()))
    by = {r["stem"]: r for r in [screen_one_anywhere(s) for s in stems]}
    bad = []
    for family, rs_name, target in retention_adoptions(
            pretend_missing, include_deleted=include_deleted):
        row = by.get(target)
        verdict = row["verdict"] if row else "NOT-IN-CORPUS"
        if verdict != "BLOCKED":
            bad.append((family, rs_name, verdict))
        elif not quiet:
            print(f"  ok  {rs_name:<42} adopted as a retention, screen says BLOCKED")
    return bad


def _crosscheck_probe() -> list:
    """The known positive. Pick a MIGRATED target the screen calls CLEAN whose
    `.rs` carries a `//!` header, pretend its case file is gone, and require the
    cross-check to go red. Without this, every green above is compatible with a
    predicate that adopts nothing at all."""
    candidates = []
    stems = sorted(set(all_stems()) | set(T19S.deleted_stems()))
    verdicts = {r["stem"]: r["verdict"] for r in [screen_one_anywhere(s) for s in stems]}
    for family, prefix in sorted(_family_prefixes().items()):
        if not prefix or family == "browser":
            continue
        for name in _corpus_names(True):
            if not name.endswith(".rs") or not name.startswith(prefix):
                continue
            stem = name[len(prefix):-3]
            if not os.path.isfile(os.path.join(CASES, family, stem + ".toml")):
                continue
            # The seed must be a target the screen calls MIGRATABLE. Seeding it
            # with a BLOCKED target proves nothing: the cross-check is supposed
            # to stay quiet about those, so the probe would "pass" against a
            # predicate that adopts nothing.
            if verdicts.get(name[:-3]) == "BLOCKED":
                continue
            try:
                text = T19S.source_text(name[:-3], quiet=True)
            except Exception:                       # noqa: BLE001
                continue
            if text.startswith("//!"):
                candidates.append((family, stem, name))
    if not candidates:
        return ["retention cross-check probe has no known positive available: no "
                "migrated, non-BLOCKED, `//!`-carrying, prefixed target exists to "
                "seed it with"]
    family, stem, name = candidates[0]
    bad = retention_crosscheck({(family, stem)}, quiet=True,
                               include_deleted=True)
    if not any(b[1] == name for b in bad):
        return [f"retention cross-check PROBE FAILED: with {family}/{stem}.toml treated "
                f"as absent, {name} must be reported as an unmigrated target "
                f"masquerading as a retention, and it was not"]
    print(f"  ok  probe: with {family}/{stem}.toml treated as absent, {name} is caught")
    return []


def main(argv: list[str]) -> int:
    rows = [screen_one(s) for s in all_stems()]
    by = {r["stem"]: r for r in rows}

    if "--retention-crosscheck" in argv:
        failures = _crosscheck_probe()
        bad = retention_crosscheck()
        for family, rs_name, verdict in bad:
            failures.append(
                f"{rs_name}: `citation_sweep.sh --family {family}` adopts this as a "
                f"whole-file RETENTION, but the screen calls it {verdict} -- an "
                f"unmigrated target masquerading as a retention, passing vacuously")
        if failures:
            print("\nRETENTION CROSS-CHECK FAILED")
            for f in failures:
                print(f"  {f}")
            return 1
        print("\nRETENTION CROSS-CHECK OK — every `.rs` the sweep adopts as a "
              "whole-file retention is a target this screen independently calls BLOCKED")
        return 0

    if "--list-clean" in argv:
        for r in rows:
            if r["verdict"] == "CLEAN":
                print(r["stem"])
        return 0

    if "--selftest" in argv:
        # The retention cross-check runs INSIDE the selftest, not beside it, so
        # the gate lane re-runs it every time rather than whoever remembers.
        # Its own known positive runs first: a comparison that has never been
        # made red is not evidence.
        failures = _crosscheck_probe()
        for family, rs_name, verdict in retention_crosscheck():
            failures.append(
                f"{rs_name}: `citation_sweep.sh --family {family}` adopts this as a "
                f"whole-file RETENTION, but the screen calls it {verdict} -- an "
                f"unmigrated target masquerading as a retention, passing vacuously")
        for stem in sorted(KNOWN_BLOCKED):
            r = by.get(stem)
            if r is None:
                failures.append(f"{stem}: in KNOWN_BLOCKED but not in the corpus")
            elif r["verdict"] == "CLEAN":
                failures.append(
                    f"{stem}: self-documented retention SCORED CLEAN -- the screen is "
                    f"validated against nothing")
            else:
                print(f"  ok  {stem:<42} {r['verdict']} via {r['blocked'][0][0]}")
        for stem in sorted(KNOWN_CLEAN):
            # `screen_one_anywhere`, not `by`: all four of these were deleted by
            # Task 19, and this control has to keep running against the same
            # four files. See `screen_one_anywhere`.
            try:
                r = screen_one_anywhere(stem)
            except Exception as exc:                # noqa: BLE001
                r = None
                failures.append(f"{stem}: in KNOWN_CLEAN but unreadable in the "
                                f"tree and at the pinned ref: {exc}")
            if r is None:
                pass
            elif r["verdict"] != "CLEAN":
                failures.append(
                    f"{stem}: already migrated, but the screen calls it {r['verdict']} "
                    f"({[b[0] for b in r['blocked']] or [a[0] for a in r['adjudicate']]}) "
                    f"-- the screen is over-blocking")
            else:
                print(f"  ok  {stem:<42} CLEAN (already migrated)")
        # A denominator that drops submodule tests is the other half of the bug.
        carriers = [r for r in rows if r["files"] > 1]
        if not carriers:
            failures.append("no #[path] carrier found -- submodule resolution is not working")
        else:
            hidden = sum(r["tests"] for r in carriers)
            print(f"  ok  {len(carriers)} `#[path]` carrier(s) resolved, {hidden} "
                  f"submodule test(s) in the denominator")
        if failures:
            print("\nSCREEN SELFTEST FAILED")
            for f in failures:
                print(f"  {f}")
            return 1
        print("\nSCREEN SELFTEST OK — every self-documented retention is caught, every "
              "already-migrated target still screens clean, and submodules are resolved")
        return 0

    clean = [r for r in rows if r["verdict"] == "CLEAN"]
    adj = [r for r in rows if r["verdict"] == "ADJUDICATE"]
    blk = [r for r in rows if r["verdict"] == "BLOCKED"]
    print(f"{len(rows)} non-browser target(s); "
          f"{sum(r['tests'] for r in rows)} #[test] fn(s) "
          f"(including {sum(r['tests'] for r in rows if r['files']>1)} behind #[path] submodules)")
    print(f"  CLEAN       {len(clean):>3} target(s), {sum(r['tests'] for r in clean):>4} test(s)")
    print(f"  ADJUDICATE  {len(adj):>3} target(s), {sum(r['tests'] for r in adj):>4} test(s)")
    print(f"  BLOCKED     {len(blk):>3} target(s), {sum(r['tests'] for r in blk):>4} test(s)")
    print("\n--- ADJUDICATE ---")
    for r in adj:
        print(f"  {r['stem']:<50} {r['tests']:>4}  {r['adjudicate'][0][0]}")
    print("\n--- BLOCKED (shape ids) ---")
    for r in blk:
        print(f"  {r['stem']:<50} {r['tests']:>4}  {','.join(b[0] for b in r['blocked'])}")
    print("\n--- CLEAN (the work list) ---")
    for r in sorted(clean, key=lambda r: -r["tests"]):
        print(f"  {r['stem']:<50} {r['tests']:>4}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
