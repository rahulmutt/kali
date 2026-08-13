#!/usr/bin/env python3
r"""Injection probe for Task 19 batch 4. Committed and runnable (U12).

Five sections. Every figure it prints is its own output, taken from inside the
loop that produced it (ruling 15's answer 1).

SECTION 1 -- THE DERIVED EXTRACTION REFUSES WHAT IT DOES NOT MODEL. Batch 4's
whole fidelity argument is that `t19b4_extract.py` is an interpreter over a
CLOSED language and raises on anything outside it -- statements, expressions and
CLAIMS alike. That argument is worth nothing unless the refusals fire, and the
extractor came out green on its first full run, which is exactly when a refusal
has to be probed. Eleven real mutations are applied to real sources; every one
must raise. The eight unmutated sources are the control.

SECTION 2 -- three arms, one poison per pair, reproducing batch 2's and batch 3's
shape so the numbers are comparable:

  arm A  `audit-case-migration.py` -- the absolute gate (rule 3)
  arm B  `fidelity.py`'s MISSING side -- an INDEPENDENT string diff
  arm C  the real binary -- the trial itself

SECTION 3 -- declination or blind spot. When arm A does not fire on a single
poisoned site, poison EVERY occurrence of the value, substrings of longer pins
included. If the audit then goes red, the single-site miss was a DECLINATION (a
coverage tool behaving correctly, the literal surviving in a sibling claim), not
a blind spot. Batch 3 measured that distinction wrong on its first run and
manufactured a blind spot that was not one.

SECTION 4 -- EXHAUSTIVE. Every case in the batch poisoned, ONE AT A TIME. Not a
sample, and not one poison per pair.

SECTION 5 -- every arm-A miss classified, not a sample of them.

**AND THE LOWER BOUND IS STATED RATHER THAN LEFT IMPLICIT.** Section 4 poisons
ONE claim per case, chosen by a priority that `_selftest_priority` gates. A case
carrying two claims has only its highest-priority one probed, so arm A's figure
is a LOWER BOUND on what the audit catches -- a much tighter one than one poison
per pair, but a lower bound, and it is not reported as anything else.
"""

from __future__ import annotations

import os
import re
import shutil
import subprocess
import sys
import tempfile
import tomllib

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
PILOT = os.path.join(REPO, "tools/task-18-browser-pilot")
sys.path.insert(0, HERE)
sys.path.insert(0, PILOT)

import fidelity  # noqa: E402
import t19b4_extract as EX  # noqa: E402
import gen_task19_batch4 as GEN  # noqa: E402

TESTS = os.path.join(REPO, "crates/kali_cli/tests")
KALI = (os.environ.get("CARGO_BIN_EXE_kali") or os.environ.get("KALI_BIN")
        or os.path.join(REPO, ".cache/cargo-target/debug/kali"))

FAILURES: list[str] = []


def check(label, ok, detail=""):
    print(f"  {'ok  ' if ok else 'FAIL'}  {label}")
    if detail:
        print(f"          {detail}")
    if not ok:
        FAILURES.append(label)


# ---------------------------------------------------------------------------
# SECTION 1 -- the refusals
# ---------------------------------------------------------------------------

MUTATIONS = [
    ("an assert! shape the table does not model",
     "growable_array_core",
     ('assert!(stderr.is_empty(), "unexpected stderr: {stderr}");',
      'assert!(output.stdout.len() > 2, "unexpected stderr");')),
    ("assert_ne!",
     "growable_array_core",
     ('assert!(stderr.is_empty(), "unexpected stderr: {stderr}");',
      'assert_ne!(stderr, "x", "unexpected stderr");')),
    ("the exit assertion removed",
     "map_iteration_runtime",
     ('assert!(!output.status.success(), "must fail closed: {output:?}");', "")),
    ("a claim written as control flow (`if !contains { panic! }`)",
     "growable_array_core",
     ('assert!(stderr.is_empty(), "unexpected stderr: {stderr}");',
      'if !stderr.contains("x") { panic!("boom") }')),
    ("a claim carried by an `.expect()`",
     "growable_array_core",
     ('assert!(stderr.is_empty(), "unexpected stderr: {stderr}");',
      'let _n = stderr.find("x").expect("x present");')),
    ("a claim on the output outside any `assert*!`",
     "growable_array_core",
     ('assert!(stderr.is_empty(), "unexpected stderr: {stderr}");',
      'let _b = output.stdout.len();')),
    ("an unmodelled `Command` method",
     "map_iteration_runtime",
     ('.arg(command)', '.env("K", "V")\n        .arg(command)')),
    ("a fixture whose body is not a resolvable literal",
     "template_literal_interpolation_runtime",
     ('run_fixture("console.log(`hello`);\\n")',
      'run_fixture(&format!("{}", std::env::var("X").unwrap()))')),
    ("an `if` whose condition is not constant at the call site",
     "for_of_object_keys_iteration",
     ('if command == "run" {', 'if source.len() > 3 {')),
    ("a `.contains` through an unmodelled receiver",
     "frozen_set_map_constructor_result",
     ('assert!(stdout.contains("ok 1"), "stdout: {stdout}");',
      'assert!(stdout.to_uppercase().contains("OK 1"), "stdout: {stdout}");')),
    ("a cross-stream disjunction over DIFFERENT needles (ruling 17's case)",
     "set_iteration_runtime",
     ('stderr.contains("E4000") || stdout.contains("E4000"),',
      'stderr.contains("E4000") || stdout.contains("E9999"),')),
]


def section1():
    print("\nSECTION 1 -- the derived extraction refuses what it does not model")
    clean = 0
    for stem in EX.STEMS:
        try:
            EX.extract(stem)
            clean += 1
        except EX.UnknownShape as exc:
            check(f"CONTROL: unmutated {stem} extracts cleanly", False, str(exc)[:200])
    check(f"control: {len(EX.STEMS)} unmutated source(s), 0 refusals",
          clean == len(EX.STEMS), f"{clean} clean")
    for label, stem, (old, new) in MUTATIONS:
        path = os.path.join(TESTS, stem + ".rs")
        original = open(path, encoding="utf-8").read()
        if old not in original:
            check(f"MUTATION ANCHOR present: {label}", False,
                  f"{stem}.rs does not contain {old[:60]!r}")
            continue
        try:
            open(path, "w", encoding="utf-8").write(original.replace(old, new, 1))
            try:
                EX.extract(stem)
                caught = False
            except EX.UnknownShape:
                caught = True
            except Exception:                                  # noqa: BLE001
                caught = True
        finally:
            open(path, "w", encoding="utf-8").write(original)
        check(f"CAUGHT  {label}", caught)


# ---------------------------------------------------------------------------
# The corpus under test
# ---------------------------------------------------------------------------

def pairs():
    """`(source stem, [(case-file path, parsed doc)])`, one entry per SOURCE."""
    out = {}
    for stem in EX.STEMS:
        for spec in GEN.build(stem):
            out.setdefault(stem, []).append(spec["path"])
    return out


def _expand(doc, path):
    """Every case as `(name, args, sources, claims)` with the matrix expanded to
    its FIRST axis cell -- one trial per case, which is what the probe poisons."""
    matrix = doc.get("matrix") or {}
    binding = {k: v[0] for k, v in matrix.items()}
    consts = dict(doc.get("constants") or {})
    binding.update(consts)

    def sub(text):
        out, rest = [], text
        while "${" in rest:
            i = rest.index("${")
            j = rest.index("}", i)
            out.append(rest[:i])
            out.append(binding.get(rest[i + 2:j], rest[i:j + 1]))
            rest = rest[j + 1:]
        return "".join(out) + rest

    srcs = {sub(k): sub(v) for k, v in (doc.get("source") or {}).items()}
    cases = []
    for c in doc.get("case") or []:
        cases.append({
            "name": c["name"],
            "args": [sub(a) for a in c.get("args", [])],
            "sources": srcs,
            "raw": c,
        })
    return cases


# ---------------------------------------------------------------------------
# Poisoning
# ---------------------------------------------------------------------------

def _poison_value(value: str) -> str | None:
    """Change the LAST alphanumeric character. CHANGE, never ADD: a poison that
    adds a character leaves the original substring intact, and a substring-checked
    claim is then still satisfied -- batch 2 §27's rule, inherited."""
    for i in range(len(value) - 1, -1, -1):
        if value[i].isalnum():
            repl = "Z" if value[i] != "Z" else "Q"
            return value[:i] + repl + value[i + 1:]
    return None


def _poisonable(case: dict):
    """The ONE claim this probe poisons in a case, by a fixed priority:

        1. a `*_contains` needle          (a real substring claim)
        2. a non-empty exact stream pin
        3. an exactly-empty stream pin    (nothing to poison inside it)
        4. the exit claim

    Returns `(kind, key, value, poisoned)` or `(None, ...)`. The priority is
    GATED by `_selftest_priority`, not merely documented: batch 3's probe stated
    this order in a docstring, implemented a different one, and filed a case that
    carries BOTH an empty pin and a needle under "nothing to poison".
    """
    raw = case["raw"]
    for key in ("stdout_contains", "stderr_contains"):
        for v in raw.get(key, []):
            p = _poison_value(v)
            if p:
                return ("needle", key, v, p)
    for key in ("stdout", "stderr"):
        v = raw.get(key)
        if isinstance(v, str) and v:
            p = _poison_value(v)
            if p:
                return ("exact", key, v, p)
    for key in ("stdout", "stderr"):
        if raw.get(key) == "":
            return ("empty", key, "", None)
    return ("exit", "exit", raw.get("exit"), None)


def _selftest_priority():
    both = {"raw": {"stdout": "", "stderr_contains": ["E5506"], "exit": "failure"}}
    only_pin = {"raw": {"stdout": "", "exit": "failure"}}
    only_exit = {"raw": {"exit": "failure"}}
    check("priority selftest: a case with BOTH an empty pin and a needle is "
          "probed on the NEEDLE", _poisonable(both)[0] == "needle",
          repr(_poisonable(both)))
    check("priority selftest: a case with only an empty pin is `empty`",
          _poisonable(only_pin)[0] == "empty", repr(_poisonable(only_pin)))
    check("priority selftest: a case with only an exit claim is `exit`",
          _poisonable(only_exit)[0] == "exit", repr(_poisonable(only_exit)))


def _write_poisoned(path: str, case_name: str, key: str, old: str, new: str,
                    all_sites: bool) -> bool:
    """Rewrite the case file with `old` replaced by `new` inside `case_name`'s
    block (or, with `all_sites`, everywhere in the file). Returns whether
    anything changed."""
    text = open(path, encoding="utf-8").read()
    olds, news = _rendered_forms(old), _rendered_forms(new)
    if all_sites:
        got = text
        for o, n in zip(olds, news):
            got = got.replace(o, n)
        if got == text:
            return False
        open(path, "w", encoding="utf-8").write(got)
        return True
    blocks = text.split("\n[[case]]\n")
    for i, b in enumerate(blocks):
        if i == 0:
            continue
        if re.search(r'^name = "%s"$' % re.escape(case_name), b, re.M):
            nb = b
            for o, n in zip(olds, news):
                if o in nb:
                    nb = nb.replace(o, n, 1)
                    break
            if nb == b:
                return False
            blocks[i] = nb
            open(path, "w", encoding="utf-8").write("\n[[case]]\n".join(blocks))
            return True
    return False


def _rendered_forms(value: str) -> list[str]:
    """Every spelling the GENERATOR could have written this value in, body only.

    Not one spelling: the generator renders a value with a newline as a TOML
    MULTI-LINE basic string (real newlines on disk) and one without as an inline
    string (`\n` escapes). A poison rewrite that knows only the escaped
    spelling silently matches nothing on every exact pin in the batch -- which is
    how this probe first reported arm A firing 0 of 196 times. The forms are
    taken from the generator's own emitters rather than restated here, so the two
    cannot disagree.
    """
    forms = [GEN.toml_inline(value)[1:-1]]
    multi = GEN.toml_str(value)
    if multi.startswith('"""'):
        forms.append(multi[len('"""\n'):-len('"""')])
    return forms


def _audit(stem, tomls):
    p = subprocess.run([sys.executable, os.path.join(REPO, "scripts/audit-case-migration.py"),
                        os.path.join(TESTS, stem + ".rs")] + tomls,
                       capture_output=True, text=True, cwd=REPO)
    return p.returncode


def _fidelity_missing(stem, tomls):
    _sc, _tc, missing, _extra = fidelity.diff([os.path.join(TESTS, stem + ".rs")],
                                              list(tomls))
    return len(missing)


def _real_binary(case, key, value) -> bool:
    """Does the REAL binary still satisfy the poisoned claim? Arm C fires when it
    does not -- which is what the trial in `cargo test --test cases` is."""
    with tempfile.TemporaryDirectory() as d:
        for k, body in case["sources"].items():
            open(os.path.join(d, k), "w").write(body)
        r = subprocess.run([KALI] + case["args"], cwd=d, capture_output=True)
    out = r.stdout.decode("utf-8", "replace")
    err = r.stderr.decode("utf-8", "replace")
    got = {"stdout": out, "stderr": err}
    if key in ("stdout_contains", "stderr_contains"):
        return value in got[key.split("_")[0]]
    if key in ("stdout", "stderr"):
        return got[key] == value
    if key == "exit":
        return (r.returncode == 0) == (value == "success")
    return True


# ---------------------------------------------------------------------------
# SECTIONS 2-5
# ---------------------------------------------------------------------------

def run_poisons(exhaustive: bool):
    per_pair = {}
    misses = []
    tot = {"n": 0, "A": 0, "B": 0, "C": 0}
    for stem, tomls in pairs().items():
        base_rc = _audit(stem, tomls)
        base_missing = _fidelity_missing(stem, tomls)
        fired = {"A": False, "B": False, "C": False}
        for path in tomls:
            doc = tomllib.load(open(path, "rb"))
            originals = open(path, encoding="utf-8").read()
            for case in _expand(doc, path):
                kind, key, value, poisoned = _poisonable(case)
                tot["n"] += 1
                a = b = c = False
                if poisoned is not None:
                    try:
                        if _write_poisoned(path, case["name"], key, value, poisoned,
                                           all_sites=False):
                            a = _audit(stem, tomls) != base_rc or _audit(stem, tomls) != 0
                            b = _fidelity_missing(stem, tomls) > base_missing
                    finally:
                        open(path, "w", encoding="utf-8").write(originals)
                    c = not _real_binary(case, key, poisoned)
                else:
                    # nothing to poison inside the value; arm C is still asked,
                    # by inverting the claim rather than mutating a character
                    if kind == "exit":
                        inv = "failure" if value == "success" else "success"
                        c = not _real_binary(case, "exit", inv)
                    else:
                        c = not _real_binary(case, key, "ZZZ-not-empty")
                tot["A"] += a
                tot["B"] += b
                tot["C"] += c
                for k, v in (("A", a), ("B", b), ("C", c)):
                    fired[k] = fired[k] or v
                if not a:
                    misses.append((stem, case["name"], kind, key, value, poisoned, path))
                if not exhaustive:
                    break
            if not exhaustive:
                break
        per_pair[stem] = fired
        if not exhaustive:
            continue
    return per_pair, misses, tot


def section2():
    print("\nSECTION 2 -- three arms, one poison per pair")
    per_pair, _misses, _tot = run_poisons(exhaustive=False)
    n = len(per_pair)
    for arm, label in (("A", "arm A (audit)"), ("B", "arm B (fidelity MISSING)"),
                       ("C", "arm C (the real binary)")):
        fired = sum(1 for v in per_pair.values() if v[arm])
        print(f"  {label:<26} fired on {fired}/{n} pairs")
    return per_pair


def section3(misses):
    print("\nSECTION 3 -- declination or blind spot: poison EVERY occurrence, "
          "substrings of longer pins included")
    out = {}
    for stem, name, kind, key, value, poisoned, path in misses:
        if poisoned is None:
            continue
        tomls = pairs()[stem]
        text = open(path, encoding="utf-8").read()
        occurrences = sum(text.count(f) for f in _rendered_forms(value))
        originals = {t: open(t, encoding="utf-8").read() for t in tomls}
        try:
            changed = _write_poisoned(path, name, key, value, poisoned, all_sites=True)
            rc = _audit(stem, tomls) if changed else 0
        finally:
            for t, o in originals.items():
                open(t, "w", encoding="utf-8").write(o)
        out[(stem, name)] = ("declination" if rc != 0 else "BLIND SPOT", occurrences,
                             changed)
    tally = {}
    for verdict, occ, changed in out.values():
        tally[verdict] = tally.get(verdict, 0) + 1
    for verdict, n in sorted(tally.items()):
        print(f"    {n:>4}  {verdict}")
    check("every all-sites poison actually rewrote something -- a rewrite that "
          "matched nothing would report a BLIND SPOT that is not one",
          all(changed for _v, _o, changed in out.values()),
          f"{sum(1 for _v, _o, c in out.values() if not c)} rewrote nothing")
    return out


def section4():
    print("\nSECTION 4 -- EXHAUSTIVE: every case poisoned, one at a time")
    per_pair, misses, tot = run_poisons(exhaustive=True)
    print(f"  {tot['n']} case(s) poisoned one at a time")
    print(f"  arm A (audit)            fired on {tot['A']:>4}/{tot['n']}")
    print(f"  arm B (fidelity MISSING) fired on {tot['B']:>4}/{tot['n']}")
    print(f"  arm C (the real binary)  fired on {tot['C']:>4}/{tot['n']}")
    return misses, tot


def section5(misses, verdicts):
    print("\nSECTION 5 -- every arm-A miss classified")
    classes = {}
    named = []
    for stem, name, kind, key, value, poisoned, path in misses:
        if kind == "exit":
            cls = "exit: out of the audit's scope by design"
        elif kind == "empty":
            cls = 'whitespace-only stream pin (`""`): nothing to poison inside it'
        else:
            verdict, occ, _changed = verdicts.get((stem, name), ("BLIND SPOT", 0, True))
            if verdict == "declination":
                cls = "declination: the literal survives in a sibling claim"
            else:
                cls = "BLIND SPOT: the audit has no claim for this value"
                named.append(f"{stem}::{name} ({key} {value!r})")
        classes[cls] = classes.get(cls, 0) + 1
    for cls, n in sorted(classes.items(), key=lambda kv: -kv[1]):
        print(f"    {n:>4}  {cls}")
    for x in named:
        print(f"  BLIND SPOT: {x}")
    return classes


def main():
    if not os.path.exists(KALI):
        print(f"PROBE CANNOT RUN -- no kali binary at {KALI}; build it or set KALI_BIN")
        return 2
    print("SECTION 0 -- the probe's own priority is gated, not documented")
    _selftest_priority()
    section1()
    section2()
    misses, tot = section4()
    verdicts = section3(misses)
    section5(misses, verdicts)
    if FAILURES:
        print(f"\nPROBE FAILED -- {len(FAILURES)} arm(s) did not behave as declared")
        for f in FAILURES:
            print(f"  {f}")
        return 1
    print(f"\nPROBE OK -- {len(MUTATIONS)} refusals fired, {tot['n']} case(s) poisoned "
          f"one at a time, arm A {tot['A']}/{tot['n']} (a LOWER bound: one claim per "
          f"case), arm C {tot['C']}/{tot['n']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
