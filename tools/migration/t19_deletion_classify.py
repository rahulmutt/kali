#!/usr/bin/env python3
"""Partition every on-disk `crates/kali_cli/tests/*.rs` into three classes.

This is the instrument Task 19's deletion step runs before deleting anything.
It answers one question per source file -- *may this `.rs` be deleted, because
every claim it makes now lives in a case file?* -- and it answers it from
evidence, per file, rather than from a name match.

WHY NOT THE PLAN'S METHOD. The plan (Task 19 Step 5) finds each stem's case
file with a stem-suffix match:

    case "$stem" in *"$b") echo "$c"; break;; esac

That is a name scan, not a classifier, and this project has already established
at review that it gets three shapes wrong:

  * A stem can suffix-match a case file that is not its migration. Measured
    here at the deletion commit's parent: the suffix scan names 66 on-disk
    sources, 8 of which NO case file claims -- `binary_stdout_runtime`,
    `imperative_core_runtime` and all six `clbg_*_runtime` targets, each of
    which merely ends with the basename of some unrelated case file. Deleting
    those 8 would have destroyed live, unmigrated coverage.
  * A stem can be PARTIALLY migrated -- a U4 trim-and-keep, where some `#[test]`
    fns moved and the rest were kept. A case file exists; the source must
    survive. Six non-browser sources are in exactly that state.
  * A stem can be a self-documented retention (spec §5.11, or a controller
    ruling), where a case file names it but the decision was to keep it.

Only resolving each case file's OWN `Migrated from` line separates a
fully-migrated stem from a survivor, and only then does the audit decide
whether the migration was total. That is what this does. Batch 8C's three-class
classifier is the model; this is the same idea against a family whose retention
marker is prose rather than the mere presence of a `//!` header (see
`retention_header` for why that difference matters and why 8C's exact predicate
could not be reused).

THE THREE CLASSES

  1. DELETE   -- at least one case file's `Migrated from` line names this
                 source, the source declares no retention, it is not a spec
                 §5.11 hand-written target, and `scripts/audit-case-migration.py`
                 reports AUDIT OK against the union of every case file that
                 claims it. Every literal claim the source makes is present in
                 the case corpus, so deleting it drops nothing.
  2. RETAINED -- claimed by a case file, but the source declares a retention
                 (a U4 trim, a §5.11 retention, an audit escalation), or the
                 audit reports a dropped claim (which is what a PARTIAL
                 migration looks like from here), or it is a §5.11 target.
  3. NOT MIGRATED -- no case file claims it. Nothing to delete against.

WHAT MAKES THIS FAIL LOUDLY RATHER THAN QUIETLY (project rule: never trust a
zero you did not try to make non-zero).

  * Every `Migrated from` occurrence in the corpus must be parsed by the claim
    regex. The count is compared, and a shortfall is a hard stop -- a claim
    this tool cannot read is a source it would silently call NOT MIGRATED.
    "Correct about every file it names" is not the same as "names the right
    set", and an unparsed claim breaks the second one only.
  * The audit's VERDICT channel is separate from its CRASH channel. A return
    code outside {0, 1}, or a 0 without `AUDIT OK` in the output, or a 1
    without `AUDIT FAILED`, is a hard stop -- this project has already shipped
    an instrument that could not tell a traceback from a clean run
    (`audit_corpus_sweep.py`, batch 5 fix round), and a deletion driven by that
    confusion would be unrecoverable.
  * A source that is claimed, declares NO retention, and FAILS its audit is a
    hard stop, not a silent RETAIN: that is a partial migration with no U3
    marker, and it needs a human ruling and a header, not a classifier's
    shrug.

REPRODUCIBILITY AFTER THE DELETION. `--ref REF` classifies the tree as it
stood at REF, materialised from git into a temp directory, touching no working
tree. That is what lets the class-1 justification in the report be re-derived
after the sources are gone -- the same lesson batch 8C paid for when six of its
generators read a working-tree source that no longer existed.

    python3 tools/migration/t19_deletion_classify.py                 # the tree
    python3 tools/migration/t19_deletion_classify.py --ref cc76f5a91 # a ref
    python3 tools/migration/t19_deletion_classify.py --selftest      # probes
    python3 tools/migration/t19_deletion_classify.py --list delete   # stems only

Exit status: 0 when the partition is complete and consistent, 1 when a hard
stop above fired.
"""

from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
import tempfile

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
TESTS_REL = "crates/kali_cli/tests"
AUDIT = os.path.join(REPO, "scripts", "audit-case-migration.py")

# Spec §5.11 keeps every one of these hand-written. Named in the plan's Task 19
# Step 6 commit message. `schema_validation` is listed there but is NOT a
# top-level target: `53a8acd146` folded it into `inprocess`, so it lives at
# `tests/inprocess/schema_validation.rs`. It is kept in this set anyway -- the
# set is the spec's, and a name that matches nothing on disk costs nothing,
# while dropping it would quietly lose the spec's intent if the fold is ever
# undone.
SPEC_511 = {
    "runtime_smoke",
    "package_corpus",
    "schema_docs",
    "node_api_surface",
    "schema_validation",
    "browser_cdp_smoke",
    "browser_harness_failing_test_propagates_failure",
    "inprocess",
}

# Not a migration source at all: `cases.rs` IS the case runner, the target every
# `.toml` in `tests/cases/` executes through. Deleting it deletes the whole
# migrated suite. Called out by name rather than left to fall into "no case file
# claims it", which is true of it but for entirely the wrong reason.
RUNNER = {"cases"}

# The `Migrated from` clause, in every shape the corpus actually uses. Verified
# exhaustive by `check_all_claims_parsed`: at the deletion commit's parent all
# 2434 occurrences match.
#
#   Migrated from tests/runtime_join.rs. ...
#   Migrated from browser_math_atan2_bracketed_root/mod_root.rs. ...
#   Migrated from `tests/soundness_url.rs`, the fn ...
#
# The optional single leading directory component is a `#[path]` submodule
# carrier's sibling directory; the basename is what identifies the top-level
# target either way.
CLAIM = re.compile(
    r"Migrated from\s+`?(?:tests/)?((?:[A-Za-z0-9_]+/)?[A-Za-z0-9_]+\.rs)`?")
CLAIM_MARKER = re.compile(r"Migrated from")

# A follow-on claim in the same clause, for a `#[path]` carrier that names its
# submodule explicitly: ", `#[path]` submodule `foo/bar.rs`".
CLAIM_SUBMODULE = re.compile(
    r"\A,\s*`#\[path\]`\s*submodule\s*`([A-Za-z0-9_/]+\.rs)`")

# WHY NOT 8C's PREDICATE. Batch 8C could read "carries a `//!` header" as "is a
# retention", because in the browser family every migrated target had had its
# header stripped and every survivor had one -- an invariant 8C's step 1
# established deliberately and then checked. That invariant DOES NOT HOLD here.
# Of the non-browser sources a case file claims, several carry an ordinary
# module docstring describing what the tests prove and nothing else
# (`arena_reclamation_runtime`, `closure_return_isolation`, `float_console_
# runtime`, `growable_array_core`, `growable_array_fail_closed`,
# `runtime_monomorphize`, `template_literal_interpolation_runtime`) -- all seven
# fully migrated, all seven audit-clean. Reusing 8C's predicate here would
# retain all seven for no reason, and "it kept too much" is exactly the failure
# mode that looks harmless and is not: it leaves duplicate coverage the whole
# task exists to remove.
#
# So the marker is the retention VOCABULARY the retentions themselves use.
# Every marker below appears in a header this project wrote to declare a
# retention, and none appears in any of the seven ordinary docstrings.
RETENTION_MARKERS = (
    "trim",                 # "TRIMMED", "U4 TRIM-AND-KEEP"
    "retention",
    "retained here",
    "kept hand-written",
    "kept 100% hand-written",
    "not migrated",
    "could not migrate",
    "audit escalation",
    "pre-trim ref:",
    "5.11",
)


def run(*args, **kw):
    return subprocess.run(args, cwd=REPO, capture_output=True, text=True, **kw)


# --------------------------------------------------------------------------
# The tree under examination: the working tree, or a ref materialised from git
# --------------------------------------------------------------------------

def materialise(ref: str) -> str:
    """`crates/kali_cli/tests` as of `ref`, in a fresh temp directory.

    Reads history, never the working tree -- so this stays answerable after the
    sources are deleted, which is the whole point of the flag.
    """
    if run("git", "rev-parse", "-q", "--verify", f"{ref}^{{commit}}").returncode:
        sys.exit(f"error: `{ref}` is not a commit reachable in this repository. "
                 "This instrument needs full history (fetch-depth: 0 in CI).")
    d = tempfile.mkdtemp(prefix="t19-classify-")
    archive = subprocess.run(
        ["git", "-C", REPO, "archive", ref, "--", TESTS_REL],
        capture_output=True)
    if archive.returncode:
        sys.exit(f"error: git archive {ref} failed: "
                 f"{archive.stderr.decode(errors='replace')}")
    tar = subprocess.run(["tar", "-x", "-C", d], input=archive.stdout,
                         capture_output=True)
    if tar.returncode:
        sys.exit(f"error: extracting {ref} failed: "
                 f"{tar.stderr.decode(errors='replace')}")
    return os.path.join(d, TESTS_REL)


# --------------------------------------------------------------------------
# Fact 1: which case files claim which source
# --------------------------------------------------------------------------

def case_files(tests: str) -> list[str]:
    out = []
    for root, _dirs, names in os.walk(os.path.join(tests, "cases")):
        out += [os.path.join(root, n) for n in names if n.endswith(".toml")]
    return sorted(out)


def claims(tests: str) -> tuple[dict[str, list[str]], int, int]:
    """`{source basename: [case files that name it]}`, plus the parse tally.

    The tally is returned rather than checked here so the caller can report it:
    a `Migrated from` this regex cannot read is a source that would be
    misclassified NOT MIGRATED, which is the one error in this tool that
    produces no symptom at all.
    """
    found: dict[str, list[str]] = {}
    seen = parsed = 0
    for path in case_files(tests):
        with open(path, encoding="utf-8") as fh:
            text = fh.read()
        rel = os.path.relpath(path, tests)
        seen += len(CLAIM_MARKER.findall(text))
        for m in CLAIM.finditer(text):
            parsed += 1
            found.setdefault(os.path.basename(m.group(1)), []).append(rel)
            sub = CLAIM_SUBMODULE.match(text[m.end():m.end() + 200])
            if sub:
                found.setdefault(os.path.basename(sub.group(1)), []).append(rel)
    return ({k: sorted(set(v)) for k, v in found.items()}, seen, parsed)


# --------------------------------------------------------------------------
# Fact 2: does the source declare a retention
# --------------------------------------------------------------------------

def docblock(text: str) -> str:
    """The leading `//!` module docstring, or "" when there is none."""
    lines = []
    for line in text.splitlines():
        stripped = line.strip()
        if stripped.startswith("//!"):
            lines.append(stripped[3:].strip())
        elif lines and not stripped:
            continue
        elif lines:
            break
    return "\n".join(lines)


def retention_header(text: str) -> list[str]:
    """Every retention marker this source's docblock carries; [] if none.

    ALL of them, not the first. These headers are long, and reporting only the
    first match makes the evidence read as weaker and narrower than it is: the
    four `soundness_*` residuals each match three or four markers independently,
    and a reader shown only `'trim'` would reasonably doubt the call. The report
    quotes this list, so it has to be the whole list.
    """
    doc = docblock(text).lower()
    return [m for m in RETENTION_MARKERS if m in doc]


# --------------------------------------------------------------------------
# Fact 3: does the audit say every claim survived
# --------------------------------------------------------------------------

class AuditAmbiguous(RuntimeError):
    pass


def audit(old: str, new: list[str]) -> tuple[str, str]:
    """`("OK"|"FAILED", output)`, or raise.

    THE VERDICT CHANNEL AND THE CRASH CHANNEL ARE DIFFERENT CHANNELS. A
    traceback out of `audit-case-migration.py` exits 1 with nothing on stdout,
    which a naive `returncode == 0` reader calls "failed" (harmless here) but
    which a naive `returncode != 1` reader would call "passed" -- and this
    project has already shipped an instrument that could not tell the two apart
    (`audit_corpus_sweep.py` compares return codes and output bytes, and a
    traceback is both). So the verdict is only accepted when the run BOTH exits
    with a verdict code AND prints the matching verdict line.
    """
    r = subprocess.run([sys.executable, AUDIT, old, *new],
                       cwd=REPO, capture_output=True, text=True)
    if r.returncode == 0 and "AUDIT OK" in r.stdout:
        return "OK", r.stdout
    if r.returncode == 1 and "AUDIT FAILED" in r.stdout:
        return "FAILED", r.stdout
    raise AuditAmbiguous(
        f"{old}: audit exited {r.returncode} without a matching verdict line. "
        f"This is a crash, not a verdict, and must not be read as either.\n"
        f"--- stdout ---\n{r.stdout}\n--- stderr ---\n{r.stderr}")


# --------------------------------------------------------------------------
# The partition
# --------------------------------------------------------------------------

def classify(tests: str) -> dict:
    claimed_by, seen, parsed = claims(tests)
    sources = sorted(n for n in os.listdir(tests) if n.endswith(".rs"))

    rows = []
    for name in sources:
        stem = name[:-3]
        with open(os.path.join(tests, name), encoding="utf-8") as fh:
            text = fh.read()
        cases = claimed_by.get(name, [])
        marker = retention_header(text)
        spec = stem in SPEC_511
        verdict = out = None
        if cases:
            verdict, out = audit(os.path.join(tests, name),
                                 [os.path.join(tests, c) for c in cases])
        rows.append({"name": name, "stem": stem, "cases": cases,
                     "retention": marker, "spec": spec,
                     "audit": verdict, "audit_output": out,
                     "tests": int(re.search(r": (\d+) #\[test\] fns", out).group(1))
                              if out else None})

    hard: list[str] = []
    if parsed != seen:
        hard.append(
            f"{seen - parsed} of {seen} `Migrated from` occurrences were not "
            "parsed by CLAIM. An unreadable claim silently demotes its source "
            "to NOT MIGRATED; refusing to publish a partition built on one.")

    delete, retain, unmigrated = [], [], []
    for row in rows:
        if row["stem"] in RUNNER:
            row["why"] = ("the case RUNNER itself, not a migration source; "
                          "every migrated `.toml` executes through it")
            unmigrated.append(row)
        elif not row["cases"]:
            row["why"] = "no case file's `Migrated from` line names it"
            unmigrated.append(row)
        elif row["spec"]:
            row["why"] = "spec §5.11 hand-written target"
            retain.append(row)
        elif row["retention"]:
            row["why"] = (f"declares a retention in its `//!` header "
                          f"(markers: {', '.join(row['retention'])}); audit "
                          f"{row['audit']}")
            retain.append(row)
        elif row["audit"] == "FAILED":
            hard.append(
                f"{row['name']}: claimed by {len(row['cases'])} case file(s), "
                "declares NO retention, and the audit reports dropped claims. "
                "That is a PARTIAL migration with no U3 marker. It needs a "
                "ruling and a header, not a classifier's default.")
            row["why"] = "UNCLASSIFIABLE — partial migration with no header"
            retain.append(row)
        else:
            row["why"] = (f"claimed by {len(row['cases'])} case file(s); "
                          f"AUDIT OK over all of them; no retention header; "
                          f"not §5.11")
            delete.append(row)

    return {"rows": rows, "delete": delete, "retain": retain,
            "unmigrated": unmigrated, "hard": hard,
            "seen": seen, "parsed": parsed, "tests_dir": tests}


# --------------------------------------------------------------------------
# The naive method, kept as a control
# --------------------------------------------------------------------------

def suffix_scan(tests: str) -> list[str]:
    """The plan's Step 5 stem-suffix match, for the report's contrast.

    A control that cannot fail is not a control: this is here so the claim
    "the name scan is wrong" is a measurement in this tool's own output rather
    than an assertion in its docstring.
    """
    bases = {os.path.basename(c)[:-5] for c in case_files(tests)}
    return sorted(n for n in os.listdir(tests) if n.endswith(".rs")
                  and any(n[:-3].endswith(b) for b in bases))


# --------------------------------------------------------------------------
# Probes
# --------------------------------------------------------------------------

def selftest() -> int:
    bad = 0

    def check(ok, label):
        nonlocal bad
        print(f"  {'ok ' if ok else 'FAIL'} {label}")
        if not ok:
            bad += 1

    # The audit reader must not turn a crash into a verdict.
    try:
        audit(os.path.join(REPO, TESTS_REL, "no_such_source_zz.rs"), [])
        check(False, "audit() on a nonexistent source raises")
    except AuditAmbiguous:
        check(True, "audit() on a nonexistent source raises, not 'OK'/'FAILED'")
    except Exception as exc:                       # noqa: BLE001
        check(False, f"audit() raised the wrong thing: {exc!r}")

    # ... and it must not accept a 0 that did not print the verdict line.
    real = subprocess.run
    try:
        class _R:
            returncode, stdout, stderr = 0, "some other output\n", ""
        subprocess.run = lambda *a, **k: _R()                      # type: ignore
        try:
            audit("x", [])
            check(False, "audit() rejects rc=0 without an `AUDIT OK` line")
        except AuditAmbiguous:
            check(True, "audit() rejects rc=0 without an `AUDIT OK` line")
    finally:
        subprocess.run = real

    # The retention predicate: fires on every marker, and on none of the
    # ordinary module docstrings this corpus actually carries.
    for marker in RETENTION_MARKERS:
        text = f"//! Task 99 {marker} something.\n\nfn main() {{}}\n"
        check(marker in retention_header(text), f"retention marker {marker!r} fires")
    ordinary = (
        "//! Runtime float values print through console with JS `String(number)`\n"
        "//! semantics. Regression for the emitter passing raw f64 into the\n"
        "//! i64-typed console imports.\n")
    check(retention_header(ordinary) == [],
          "an ordinary module docstring is not read as a retention")
    check(retention_header("fn main() {}\n") == [], "no docblock is not a retention")
    # A retention marker BELOW the docblock is not a retention declaration.
    check(retention_header("//! Ordinary.\n\nfn f() { /* TRIMMED */ }\n") == [],
          "a marker outside the `//!` docblock does not fire")

    # The claim regex, on each shape the corpus uses.
    for text, want in [
        ('rationale = """Migrated from tests/runtime_join.rs. x"""', "runtime_join.rs"),
        ('rationale = """Migrated from browser_x/mod_root.rs. y"""', "mod_root.rs"),
        ('rationale = """Migrated from `tests/soundness_url.rs`, the fn z"""',
         "soundness_url.rs"),
    ]:
        m = CLAIM.search(text)
        check(m is not None and os.path.basename(m.group(1)) == want,
              f"CLAIM parses {want}")
    check(CLAIM.search("no claim here") is None, "CLAIM does not match arbitrary prose")

    print("SELFTEST OK" if not bad else f"SELFTEST FAILED — {bad} probe(s)")
    return 1 if bad else 0


# --------------------------------------------------------------------------

def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--ref", help="classify the tree as of this commit")
    ap.add_argument("--selftest", action="store_true")
    ap.add_argument("--list", choices=["delete", "retain", "unmigrated"],
                    help="print only that class's stems, one per line")
    args = ap.parse_args()

    if args.selftest:
        return selftest()

    tests = materialise(args.ref) if args.ref else os.path.join(REPO, TESTS_REL)
    try:
        result = classify(tests)
    except AuditAmbiguous as exc:
        print(f"HARD STOP — {exc}")
        return 1

    if args.list:
        for row in result[{"delete": "delete", "retain": "retain",
                           "unmigrated": "unmigrated"}[args.list]]:
            print(row["name"])
        return 0 if not result["hard"] else 1

    at = args.ref or "the working tree"
    print(f"=== Task 19 deletion classification at {at} ===")
    print(f"{len(result['rows'])} top-level `.rs` under {TESTS_REL}; "
          f"{len(case_files(tests))} case files; "
          f"{result['parsed']}/{result['seen']} `Migrated from` occurrences parsed")

    naive = suffix_scan(tests)
    claimed = {r["name"] for r in result["rows"] if r["cases"]}
    print(f"\nCONTROL — the plan's stem-suffix scan names {len(naive)}; "
          f"{len(set(naive) - claimed)} of those are claimed by NO case file "
          f"and would be deleted wrongly: "
          f"{', '.join(sorted(set(naive) - claimed)) or '(none)'}")

    for title, key in [("1. DELETE — fully migrated", "delete"),
                       ("2. RETAINED — claimed, but kept", "retain"),
                       ("3. NOT MIGRATED — no case file claims it", "unmigrated")]:
        rows = result[key]
        print(f"\n--- CLASS {title}: {len(rows)} ---")
        for row in rows:
            fns = f"{row['tests']} #[test] fns, " if row["tests"] else ""
            print(f"  {row['name']}: {fns}{row['why']}")
            if key == "delete":
                print(f"      case files: {', '.join(row['cases'])}")

    print(f"\nSUMMARY delete={len(result['delete'])} "
          f"retain={len(result['retain'])} "
          f"not_migrated={len(result['unmigrated'])} "
          f"total={len(result['rows'])}")

    if result["hard"]:
        print(f"\nHARD STOP — {len(result['hard'])} condition(s); delete nothing:")
        for note in result["hard"]:
            print(f"  {note}")
        return 1
    print("CLASSIFICATION OK — the partition is complete and consistent")
    return 0


if __name__ == "__main__":
    sys.exit(main())
