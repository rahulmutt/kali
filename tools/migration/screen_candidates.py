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

Usage:
  screen_candidates.py                 # full report
  screen_candidates.py --selftest      # gate: nonzero if ground truth regresses
  screen_candidates.py --list-clean    # the work list, one stem per line
"""

from __future__ import annotations

import os
import re
import sys

TESTS = os.path.join(
    os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))),
    "crates/kali_cli/tests",
)

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

ADJUDICABLE = [
    ("S26_combined_streams",
     r"push_str\(&String::from_utf8_lossy\(&output\.stderr\)|combined\.push_str",
     "asserts against stdout+stderr CONCATENATED; resolvable per rule 11 for a "
     "presence claim, but an absence claim may not be narrowed (rule 2)"),
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
    tests = 0
    for f in files:
        try:
            t = open(f, encoding="utf-8", errors="replace").read()
        except OSError:
            continue
        blobs.append(t)
        tests += len(TEST_FN.findall(t))
    blob = "\n".join(blobs)

    blocked = [(sid, why) for sid, rx, why in BLOCKING if re.search(rx, blob)]
    adjud = [(sid, why) for sid, rx, why in ADJUDICABLE if re.search(rx, blob)]
    if header_says_retained(blobs[0] if blobs else ""):
        blocked.insert(0, ("S27_self_documented",
                           "the file's own `//!` header states it could not be migrated"))

    verdict = "BLOCKED" if blocked else ("ADJUDICATE" if adjud else "CLEAN")
    return {"stem": stem, "tests": tests, "files": len(files),
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


def main(argv: list[str]) -> int:
    rows = [screen_one(s) for s in all_stems()]
    by = {r["stem"]: r for r in rows}

    if "--list-clean" in argv:
        for r in rows:
            if r["verdict"] == "CLEAN":
                print(r["stem"])
        return 0

    if "--selftest" in argv:
        failures = []
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
            r = by.get(stem)
            if r is None:
                failures.append(f"{stem}: in KNOWN_CLEAN but not in the corpus")
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
