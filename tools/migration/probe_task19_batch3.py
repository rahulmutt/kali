#!/usr/bin/env python3
r"""Injection probe for Task 19 batch 3 -- what the gates on this family actually catch.

RULING 15's LAST PARAGRAPH IS WHY THIS EXISTS: "a green suite is not evidence for
a gate change ... verify only with an injection probe showing the check still
fires on the thing it exists to catch." Batch 2 added that a SINGLE-ARMED probe
reports a comfortable green and measures the wrong gate, so this has three arms
and records WHICH fired rather than requiring a particular one.

SECTION 1 -- THE DERIVED EXTRACTION REFUSES. Batch 3's whole fidelity argument
is that `t19b3_extract` raises on an assertion shape it does not model rather
than skipping it. A refusal nobody has made fire is a refusal nobody has
measured, so six real mutations are applied to real sources and every one must
raise, with the unmutated control clean.

SECTION 2 -- THREE ARMS, ONE POISON PER PAIR.

  arm A  `audit-case-migration.py`  -- forward literal coverage
  arm B  `fidelity.py`'s MISSING side -- the independent string diff
  arm C  the real `kali` binary     -- U9's live check, run directly on the
                                       poisoned case's own fixture

ONE POISON PER PAIR MAKES ARM A A LOWER BOUND, NOT A MEASUREMENT. That sentence
is batch 2's and it is repeated here because it is the only honest reading: a
coverage tool that misses one poisoned copy of a literal still present elsewhere
is behaving correctly, not blind.

SECTION 3 -- THE DECLINATION/BLIND-SPOT DISCRIMINATOR (batch 2 §27). Poison
EVERY occurrence of the chosen value, not one. If the audit then fires, the
single-site miss was a coverage tool behaving correctly; if it still does not,
the audit has no claim for that value at all and the miss is a real blind spot.

POISON RULE, inherited: a poison for a substring-checked claim must CHANGE a
character, never ADD one. Appending `X` leaves the source literal a substring of
the poisoned value, so the audit's coverage arm is satisfied by the poisoned
file and the row reads "untestable as poisoned".

Every file is restored in a `finally`; the probe refuses to start on a dirty
case tree.
"""

from __future__ import annotations

import os
import re
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, os.path.join(REPO, "tools/task-18-browser-pilot"))
sys.path.insert(0, HERE)

import fidelity  # noqa: E402
import t19b3_extract as X  # noqa: E402
from case_emit import cargo_target_dir  # noqa: E402
from gen_task19_batch3 import FILES  # noqa: E402
from toml_emit import toml_string  # noqa: E402

TESTS = os.path.join(REPO, "crates/kali_cli/tests")
CASES = os.path.join(TESTS, "cases")
AUDIT = os.path.join(REPO, "scripts/audit-case-migration.py")


def _kali():
    path = os.path.join(cargo_target_dir(), "debug", "kali")
    if not os.path.exists(path):
        raise AssertionError(
            f"no built `kali` at {path}. Arm C is U9's LIVE check and will not be "
            "faked: build it with `cargo build -p kali_cli --bin kali` and re-run.")
    return path


# --------------------------------------------------------------------------
# Section 1 -- the extractor refuses
# --------------------------------------------------------------------------

MUTATIONS = [
    ("an assert! shape the table does not model", "runtime_join",
     '    assert_eq!(String::from_utf8_lossy(&out.stdout), "xxx\\n");',
     '    assert!(out.stdout.len() > 2, "len");'),
    ("assert_ne!", "runtime_join",
     '    assert_eq!(String::from_utf8_lossy(&out.stdout), "xxx\\n");',
     '    assert_ne!(String::from_utf8_lossy(&out.stdout), "q");'),
    ("a .contains against STDOUT, not stderr", "runtime_join",
     '    assert_eq!(String::from_utf8_lossy(&out.stdout), "xxx\\n");',
     '    assert!(String::from_utf8_lossy(&out.stdout).contains("x"), "m");'),
    ("the exit assertion removed", "runtime_join",
     '    assert!(\n        out.status.success(),\n        "stderr: {}",\n'
     '        String::from_utf8_lossy(&out.stderr)\n    );\n'
     '    assert_eq!(String::from_utf8_lossy(&out.stdout), "xxx\\n");',
     '    assert_eq!(String::from_utf8_lossy(&out.stdout), "xxx\\n");'),
    ("a fixture built by format! instead of a literal", "runtime_ternary",
     '    let out = run_source("let a = 2;\\nconsole.log(a == 1 ? 10 : a == 2 ? 20 : 30);\\n");',
     '    let out = run_source(&format!("let a = {};", 2));'),
    ("two run_source calls in one #[test]", "runtime_ternary",
     '    let out = run_source("let a = 2;\\nconsole.log(a == 1 ? 10 : a == 2 ? 20 : 30);\\n");',
     '    let out = run_source("a");\n    let _ = run_source("b");'),
]


def _extract_all(stem, text):
    problems = []
    for f in X.test_fns(text):
        try:
            X.fixture_of(stem, f)
            key = (stem, f["name"])
            X.claims_of(stem, f,
                        computed_stdout="<declared>" if key in X.COMPUTED else None)
        except X.UnknownShape as e:
            problems.append(str(e))
    return problems


def section1():
    print("SECTION 1 -- the derived extraction refuses what it does not model")
    bad = 0
    for stem in X.STEMS:
        if _extract_all(stem, X.source(stem)):
            print(f"  CONTROL FAILED: unmutated {stem}.rs already raises")
            bad += 1
    print(f"  control: {len(X.STEMS)} unmutated source(s), 0 refusals")
    for label, stem, old, new in MUTATIONS:
        text = X.source(stem)
        if old not in text:
            print(f"  PROBE STALE: the anchor for {label!r} is no longer in {stem}.rs")
            bad += 1
            continue
        problems = _extract_all(stem, text.replace(old, new, 1))
        state = "CAUGHT" if problems else "MISSED"
        print(f"  {state:<7} {label}")
        if not problems:
            bad += 1
    return bad


# --------------------------------------------------------------------------
# Section 2/3 -- the injection probe
# --------------------------------------------------------------------------

def _case_blocks(text):
    """`[(name, start, end)]` over the `[[case]]` blocks of a rendered file."""
    out = []
    marks = [m.start() for m in re.finditer(r"^\[\[case\]\]$", text, re.M)]
    for i, s in enumerate(marks):
        e = marks[i + 1] if i + 1 < len(marks) else len(text)
        name = re.search(r'^name = "([^"]+)"', text[s:e], re.M).group(1)
        out.append((name, s, e))
    return out


def _poison_value(value):
    """Change a character; never add one (batch 2 §27's corrected rule)."""
    for i in range(len(value) - 1, -1, -1):
        c = value[i]
        if c.isalnum():
            repl = "Q" if c != "Q" else "Z"
            return value[:i] + repl + value[i + 1:]
    raise AssertionError(f"nothing poisonable in {value!r}")


def _pick(text):
    """The claim this pair is probed on: the first exact `stdout` pin with a
    poisonable character, else the first `stderr_contains` needle. Chosen by a
    rule rather than per file, so nothing is selected for being easy to catch."""
    for name, s, e in _case_blocks(text):
        block = text[s:e]
        m = re.search(r'^stdout = (".*")$', block, re.M)
        if m:
            raw = m.group(1)
            value = re.sub(r'\\n', '\n', raw[1:-1]).replace('\\"', '"').replace("\\\\", "\\")
            if any(c.isalnum() for c in value):
                return name, "stdout", value, m.start() + s, m.end() + s
    for name, s, e in _case_blocks(text):
        block = text[s:e]
        m = re.search(r'^stderr_contains = \["([^"]+)"\]$', block, re.M)
        if m:
            return name, "stderr_contains", m.group(1), m.start() + s, m.end() + s
    raise AssertionError("no poisonable claim in this file")


def _rewrite(text, lo, hi, key, value):
    return text[:lo] + f"{key} = {toml_string(value, multiline=False)}" + text[hi:]


def _audit(stem, toml_path):
    return subprocess.run(
        [sys.executable, AUDIT, stem + ".rs", os.path.relpath(toml_path, TESTS)],
        cwd=TESTS, capture_output=True, text=True).returncode


def _fidelity_missing(stem, toml_path):
    _sc, _tc, missing, _extra = fidelity.diff(
        [os.path.join(TESTS, stem + ".rs")], [toml_path])
    return missing


def _arm_c(stem, toml_path, case_name, key, poisoned):
    """U9's live arm: run the real binary on THIS case's own fixture and ask
    whether the poisoned claim still holds. Reproduces what the runner asserts,
    without a 30-second suite rebuild per poison."""
    import tomllib
    doc = tomllib.load(open(toml_path, "rb"))
    case = next(c for c in doc["case"] if c["name"] == case_name)
    argv = case["args"]
    d = tempfile.mkdtemp(prefix="t19b3-probe-")
    try:
        for fname, body in doc["source"].items():
            with open(os.path.join(d, fname), "w") as fh:
                fh.write(body)
        p = subprocess.run([_kali()] + argv, cwd=d, capture_output=True, text=True)
        if key == "stdout":
            return p.stdout != poisoned
        return poisoned not in p.stderr
    finally:
        shutil.rmtree(d, ignore_errors=True)


def section2():
    print("\nSECTION 2 -- three arms, one poison per pair")
    fired = {"A": 0, "B": 0, "C": 0}
    rows, bad = [], 0
    for family, toml, stem, _subject in FILES:
        toml_path = os.path.join(CASES, family, toml + ".toml")
        original = open(toml_path).read()
        name, key, value, lo, hi = _pick(original)
        poisoned = _poison_value(value)
        base_missing = _fidelity_missing(stem, toml_path)
        try:
            with open(toml_path, "w") as fh:
                fh.write(_rewrite(original, lo, hi, key, poisoned))
            a = _audit(stem, toml_path) != 0
            b = len(_fidelity_missing(stem, toml_path)) > len(base_missing)
            c = _arm_c(stem, toml_path, name, key, poisoned)
        finally:
            with open(toml_path, "w") as fh:
                fh.write(original)
        assert open(toml_path).read() == original, f"{toml_path} not restored"
        for k, v in (("A", a), ("B", b), ("C", c)):
            fired[k] += bool(v)
        if not c:
            bad += 1
        rows.append((f"{family}/{toml}", name, key, a, b, c))
        print(f"  {family}/{toml:<24} poisoned {key:<16} of {name[:44]:<44} "
              f"A={'Y' if a else '.'} B={'Y' if b else '.'} C={'Y' if c else '.'}")
    n = len(FILES)
    print(f"\n  arm A (audit)            fired on {fired['A']}/{n} pairs")
    print(f"  arm B (fidelity MISSING) fired on {fired['B']}/{n} pairs")
    print(f"  arm C (the real binary)  fired on {fired['C']}/{n} pairs")
    return bad, rows


def section3(rows):
    """Poison EVERY OCCURRENCE, and re-ask arm A.

    "Every occurrence" means every place the literal appears in the case file,
    INCLUDING inside a longer pin -- not just the sites that spell it as a whole
    value. That distinction is the whole discriminator and getting it wrong
    manufactures a blind spot out of a declination: `runtime/substring_length`'s
    `"CCAA\\n"` occurs once as a whole `stdout` value and twice more inside
    `"CCAA\\nGGC\\nAATT\\n"` and `"TTGGCCAA\\n"`, so poisoning only the first
    leaves the audit's SUBSTRING coverage arm satisfied by the other two -- a
    coverage tool behaving exactly correctly. rc=1 after poisoning all of them
    means the single-site miss was a declination; rc=0 means the audit has no
    claim for that value at all.
    """
    print("\nSECTION 3 -- declination or blind spot: poison EVERY occurrence, "
          "substrings of longer pins included")
    out = []
    for (pair, _name, _key, a, _b, _c) in rows:
        family, toml = pair.split("/")
        stem = next(s for f, t, s, _x in FILES if (f, t) == (family, toml))
        toml_path = os.path.join(CASES, family, toml + ".toml")
        original = open(toml_path).read()
        _n, _k, value, _lo, _hi = _pick(original)
        poisoned = _poison_value(value)
        spelled = toml_string(value, multiline=False)[1:-1]
        spelled_bad = toml_string(poisoned, multiline=False)[1:-1]
        sites = original.count(spelled)
        try:
            with open(toml_path, "w") as fh:
                fh.write(original.replace(spelled, spelled_bad))
            rc = _audit(stem, toml_path)
        finally:
            with open(toml_path, "w") as fh:
                fh.write(original)
        verdict = ("arm A already fired on one site" if a
                   else ("declination" if rc else "BLIND SPOT"))
        out.append((pair, sites, rc, verdict))
        print(f"  {pair:<32} occurrences={sites:<3} all-poisoned audit rc={rc}  "
              f"{verdict}")
    return out


def _poisonable(text, name, s, e):
    """The claim this case is probed on, and how to poison it.

    Priority is fixed rather than chosen per case: an exact `stdout` pin with a
    poisonable character, then a `stderr_contains` needle, then the empty-stdout
    pin, then the exit status. Every case in this family has at least one, which
    is why section 4 can be exhaustive.
    """
    block = text[s:e]
    m = re.search(r'^stdout = (".*")$', block, re.M)
    if m:
        raw = m.group(1)
        value = re.sub(r'\\n', '\n', raw[1:-1]).replace('\\"', '"').replace("\\\\", "\\")
        if any(c.isalnum() for c in value):
            return "stdout", value, _poison_value(value), m.start() + s, m.end() + s
        return "stdout(empty)", value, "Q\n", m.start() + s, m.end() + s
    m = re.search(r'^stderr_contains = \["([^"]+)"\]$', block, re.M)
    if m:
        v = m.group(1)
        return ("stderr_contains", v, _poison_value(v),
                m.start() + s, m.end() + s)
    m = re.search(r'^exit = "(\w+)"$', block, re.M)
    flip = {"success": "failure", "failure": "success"}[m.group(1)]
    return "exit", m.group(1), flip, m.start() + s, m.end() + s


def _rewrite_claim(text, lo, hi, key, value):
    key = {"stdout(empty)": "stdout"}.get(key, key)
    if key == "stderr_contains":
        return text[:lo] + f"stderr_contains = [{toml_string(value, multiline=False)}]" + text[hi:]
    return text[:lo] + f"{key} = {toml_string(value, multiline=False)}" + text[hi:]


def _arm_c_generic(doc_source, argv, key, poisoned):
    d = tempfile.mkdtemp(prefix="t19b3-probe-")
    try:
        for fname, body in doc_source.items():
            with open(os.path.join(d, fname), "w") as fh:
                fh.write(body)
        p = subprocess.run([_kali()] + argv, cwd=d, capture_output=True, text=True)
        if key in ("stdout", "stdout(empty)"):
            return p.stdout != poisoned
        if key == "stderr_contains":
            return poisoned not in p.stderr
        return (p.returncode == 0) != (poisoned == "success")
    finally:
        shutil.rmtree(d, ignore_errors=True)


def section4():
    """EXHAUSTIVE: poison every case, one at a time, and record which arm fires.

    Batch 2 reported arm A at 9/17 and said in as many words that one poison per
    pair is a LOWER BOUND rather than a measurement. This is the same probe run
    over every case in the batch instead of one per file -- 123 poisons -- so the
    number it reports is what the gates catch on this family, not a sample of it.
    """
    import tomllib
    print("\nSECTION 4 -- EXHAUSTIVE: every case poisoned, one at a time")
    tot = {"A": 0, "B": 0, "C": 0, "n": 0}
    per_file = []
    misses = []
    arm_a_misses = []
    for family, toml, stem, _subject in FILES:
        toml_path = os.path.join(CASES, family, toml + ".toml")
        original = open(toml_path).read()
        doc = tomllib.load(open(toml_path, "rb"))
        by_name = {c["name"]: c for c in doc["case"]}
        base_missing = _fidelity_missing(stem, toml_path)
        f = {"A": 0, "B": 0, "C": 0, "n": 0}
        try:
            for name, s, e in _case_blocks(original):
                key, _value, poisoned, lo, hi = _poisonable(original, name, s, e)
                with open(toml_path, "w") as fh:
                    fh.write(_rewrite_claim(original, lo, hi, key, poisoned))
                a = _audit(stem, toml_path) != 0
                b = len(_fidelity_missing(stem, toml_path)) > len(base_missing)
                c = _arm_c_generic(doc["source"], by_name[name]["args"], key, poisoned)
                for k, v in (("A", a), ("B", b), ("C", c)):
                    f[k] += bool(v)
                    tot[k] += bool(v)
                f["n"] += 1
                tot["n"] += 1
                if not (a or b or c):
                    misses.append(f"{family}/{toml}::{name} ({key})")
                if not a:
                    arm_a_misses.append((family, toml, stem, name, key, _value))
        finally:
            with open(toml_path, "w") as fh:
                fh.write(original)
        per_file.append((f"{family}/{toml}", f))
        print(f"  {family}/{toml:<24} n={f['n']:<4} A={f['A']:<4} B={f['B']:<4} C={f['C']}")
    print(f"\n  {tot['n']} case(s) poisoned one at a time")
    print(f"  arm A (audit)            fired on {tot['A']}/{tot['n']}")
    print(f"  arm B (fidelity MISSING) fired on {tot['B']}/{tot['n']}")
    print(f"  arm C (the real binary)  fired on {tot['C']}/{tot['n']}")
    for m in misses:
        print(f"  UNCAUGHT BY EVERY ARM: {m}")
    return len(misses), tot, per_file, arm_a_misses


def section5(arm_a_misses):
    """Classify EVERY arm-A miss, not a sample: declination or blind spot.

    Batch 2 classified nine misses by hand. This runs the discriminator over all
    of them: poison every occurrence of the literal in the file (substrings of
    longer pins included) and re-ask the audit. Two classes need no run because
    they are structural:

      * an `exit` poison -- the audit declares exit status out of scope for a
        literal-coverage tool, so it can never fire on one. Not a blind spot: a
        gate that says what it does not cover is not blind, and `exit` is what
        covers it.
      * a whitespace-only `stdout` poison -- `""` (12 cases) and `"\n"` (one)
        have no alphanumeric character to change, and both are in
        `fidelity.BORING`, so neither literal extractor has anything to find.
    """
    print("\nSECTION 5 -- every arm-A miss classified")
    classes = {}
    detail = []
    for family, toml, stem, name, key, value in arm_a_misses:
        if key == "exit":
            klass = "exit: out of the audit's scope by design"
        elif key == "stdout(empty)":
            klass = ("whitespace-only stdout pin (`\"\"` or `\"\\n\"`): no "
                     "alphanumeric character to poison, and the value is "
                     "BORING-excluded from both extractors")
        else:
            toml_path = os.path.join(CASES, family, toml + ".toml")
            original = open(toml_path).read()
            spelled = toml_string(value, multiline=False)[1:-1]
            bad = toml_string(_poison_value(value), multiline=False)[1:-1]
            try:
                with open(toml_path, "w") as fh:
                    fh.write(original.replace(spelled, bad))
                rc = _audit(stem, toml_path)
            finally:
                with open(toml_path, "w") as fh:
                    fh.write(original)
            klass = ("declination: the literal survives in a sibling claim"
                     if rc else "BLIND SPOT: the audit has no claim for this value")
        classes[klass] = classes.get(klass, 0) + 1
        detail.append((f"{family}/{toml}::{name}", klass))
    for klass, n in sorted(classes.items(), key=lambda kv: -kv[1]):
        print(f"  {n:>4}  {klass}")
    blind = [d for d in detail if d[1].startswith("BLIND SPOT")]
    for pair, _k in blind:
        print(f"  BLIND SPOT: {pair}")
    return classes, detail


def main():
    dirty = subprocess.run(["git", "status", "--porcelain", "--", "crates/kali_cli/tests/cases"],
                           cwd=REPO, capture_output=True, text=True).stdout
    if dirty.strip():
        print("REFUSING TO RUN: cases/ is dirty; this probe edits case files in place "
              "and restores them, and it will not do that over uncommitted work.\n"
              + dirty)
        return 2
    bad = section1()
    bad2, rows = section2()
    section3(rows)
    bad4, tot = 0, None
    if "--quick" not in sys.argv[1:]:
        bad4, tot, _per, arm_a_misses = section4()
        section5(arm_a_misses)
    dirty = subprocess.run(["git", "status", "--porcelain", "--", "crates/kali_cli/tests/cases"],
                           cwd=REPO, capture_output=True, text=True).stdout
    if dirty.strip():
        print("\nPROBE FAILED -- case files were not restored:\n" + dirty)
        return 1
    if bad or bad2 or bad4:
        print(f"\nPROBE FAILED -- {bad} extraction refusal(s) missed, {bad2} pair(s) "
              f"whose poison no arm caught, {bad4} exhaustive poison(s) no arm caught")
        return 1
    tail = (f", and all {tot['n']} cases poisoned exhaustively with arm A at "
            f"{tot['A']}/{tot['n']} and arm C at {tot['C']}/{tot['n']}") if tot else ""
    print(f"\nPROBE OK -- {len(MUTATIONS)} extraction refusals all fired, "
          f"{len(FILES)} pairs each caught by at least the live arm{tail}, "
          "every file restored")
    return 0


if __name__ == "__main__":
    sys.exit(main())
