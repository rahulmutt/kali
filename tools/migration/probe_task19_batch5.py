#!/usr/bin/env python3
r"""Injection probe for Task 19 batch 5. Committed and runnable (U12).

Five sections. Every figure it prints is its own output, taken from inside the
loop that produced it (ruling 15's answer 1).

SECTION 0 -- THE PROBE'S OWN PRIORITY IS GATED, NOT DOCUMENTED. Batch 3's probe
stated a priority in a docstring and implemented a different one. This one's is
asserted against synthetic cases before anything else runs.

SECTION 1 -- THE DERIVED EXTRACTION REFUSES WHAT IT DOES NOT MODEL. This
batch's whole fidelity argument is that `t19b5_extract.py` is an interpreter
over a CLOSED language and raises on anything outside it -- statements,
expressions and CLAIMS alike. That argument is worth nothing unless the refusals
fire, and the extractor came out green on its first full run, which is exactly
when a refusal has to be probed. Real mutations are applied to real sources;
every one must raise. The seven unmutated sources are the control.

SECTION 2 -- three arms, one poison per pair, so the numbers are comparable
with batches 2, 3 and 4:

  arm A  `audit-case-migration.py` -- the absolute gate (rule 3)
  arm B  `fidelity.py`'s MISSING side -- an INDEPENDENT string diff
  arm C  the real binary -- the trial itself

SECTION 3 -- declination or blind spot. When arm A does not fire on a single
poisoned site, poison EVERY occurrence of the value. If the audit then goes red,
the single-site miss was a DECLINATION (a coverage tool behaving correctly, the
literal surviving in a sibling claim), not a blind spot. Batch 3 measured that
distinction wrong on its first run and manufactured a blind spot that was not
one; and an all-sites rewrite that changes NO BYTES is asserted against, because
batch 4's probe classified one of those as a blind spot.

SECTION 4 -- EXHAUSTIVE. Every case in the batch poisoned, ONE AT A TIME. Not a
sample, and not one poison per pair.

SECTION 5 -- every arm-A miss classified, not a sample of them.

**AND THE LOWER BOUND IS STATED RATHER THAN LEFT IMPLICIT.** Section 4 poisons
ONE claim per case, chosen by the priority section 0 gates. A case carrying
several claims has only its highest-priority one probed, so arm A's figure is a
LOWER BOUND on what the audit catches -- a much tighter one than one poison per
pair, but a lower bound, and it is not reported as anything else.

THE TRIMMED PAIR IS AUDITED AGAINST ITS CORRECT SIDE. `for_of_array_iteration_
spread` is a U4 trim, and `audit-case-migration.py` is a FORWARD coverage gate,
so ruling 19 puts its correct left-hand side at the MIGRATED COMPLEMENT. Running
arm A against the post-trim file would report rc=1 on every poison and every
control alike -- a probe arm that fires unconditionally measures nothing.

**THIS SCRIPT MUTATES REAL CASE FILES IN PLACE** (poison, measure, restore), so
it is unsafe to run concurrently with anything else that reads or writes them.
Run it serially, never beside a `test-gate.sh` invocation.
"""

from __future__ import annotations

import collections
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
import gen_task19_batch5 as GEN  # noqa: E402
import t19b5_extract as EX  # noqa: E402
from toml_emit import toml_string  # noqa: E402

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
     "thread_topology_json",
     ('assert_eq!(value["totalInstances"], 0);',
      'assert!(value["totalInstances"].is_i64());')),
    ("assert_ne!",
     "thread_topology_json",
     ('assert_eq!(value["totalInstances"], 0);',
      'assert_ne!(value["totalInstances"], 1);')),
    ("the ONLY assertion removed, leaving a command with no claim",
     "runtime_forin",
     ('assert!(!out.status.success(), "-c must fail closed");', "")),
    ("a claim written as control flow (`if !contains { panic! }`)",
     "wrapped_call_targets_wrappers",
     ('assert!(stdout.contains("7\\n7\\n"), "stdout: {stdout}");',
      'if !stdout.contains("7\\n7\\n") { panic!("boom") }')),
    ("a claim carried by an `.expect()`",
     "wrapped_call_targets_wrappers",
     ('assert!(stdout.contains("7\\n7\\n"), "stdout: {stdout}");',
      'let _n = stdout.find("7\\n7\\n").expect("present");')),
    ("a claim on the output outside any `assert*!`",
     "wrapped_call_targets_wrappers",
     ('assert!(stdout.contains("7\\n7\\n"), "stdout: {stdout}");',
      'let _b = output.stdout.len();')),
    ("an unmodelled `Command` method",
     "parse_float_static_ascii",
     ('.arg("run")', '.stdin(std::process::Stdio::null())\n        .arg("run")')),
    ("an unmodelled json member",
     "thread_topology_json",
     ('assert_eq!(value["totalInstances"], 0);',
      'assert_eq!(value["totalInstances"].as_u64_or_zero(), 0);')),
    ("a `.contains` through an unmodelled receiver",
     "wrapped_call_targets_wrappers",
     ('assert!(stdout.contains("7\\n7\\n"), "stdout: {stdout}");',
      'assert!(stdout.to_uppercase().contains("7\\n7\\n"), "stdout: {stdout}");')),
    ("an `if` whose condition is not constant at the call site",
     "logical_assignment_wrapped_local_binding",
     ('if json_output {\n        let json: Value',
      'if source.len() > 3 {\n        let json: Value')),
    ("a `format!`-built fixture with no capture (rules 8/9)",
     "runtime_forin",
     ('let out = forin_leak_case("var x = 0; for (var c in tab) { return (x += c); } return 0;");',
      'let out = forin_leak_case("var x = 1; for (var c in tab) { return (x += c); } return 0;");')),
    ("a fixture-self-inspection `.matches` on builder text (ruling 10)",
     "parse_float_static_ascii",
     ('    assert_eq!(json["schemaVersion"], 1);\n    assert_eq!(json["command"], "check");',
      '    assert_eq!(supported_source().matches("parseFloat").count(), 8);')),
    ("a disjunction over shapes the table does not model",
     "runtime_forin",
     ('!out.status.success() || String::from_utf8_lossy(&out.stdout) == "a\\n",\n        "sequence-expression key must fail closed;',
      '!out.status.success() || out.stdout.len() == 2,\n        "sequence-expression key must fail closed;')),
]


def section1():
    print("\nSECTION 1 -- the derived extraction refuses what it does not model")
    clean = 0
    for stem in EX.STEMS:
        try:
            EX.extract(stem)
            clean += 1
        except Exception as exc:                               # noqa: BLE001
            check(f"CONTROL: unmutated {stem} extracts cleanly", False, str(exc)[:200])
    check(f"control: {len(EX.STEMS)} unmutated source(s), 0 refusals",
          clean == len(EX.STEMS), f"{clean} clean")
    for label, stem, (old, new) in MUTATIONS:
        path = os.path.join(TESTS, stem + ".rs")
        with open(path, encoding="utf-8") as f:
            original = f.read()
        # ONE stem in this batch is read at a PINNED REF by the extractor:
        # `for_of_array_iteration_spread`, the U4 trim, is the sole member of
        # `EX.PRE_TRIM`, so a working-tree mutation to THAT file would not reach
        # the extractor and no mutation below targets it. `runtime_forin` is
        # NOT pinned -- it is read from the working tree like every other stem,
        # which is why the three mutations that target it are live. (The earlier
        # spelling of this comment named both files as pinned, which would have
        # told a reader those three mutations were inert.) The anchor check
        # below is what proves each mutation landed either way.
        if old not in original:
            check(f"MUTATION ANCHOR present: {label}", False,
                  f"{stem}.rs does not contain {old[:60]!r}")
            continue
        try:
            with open(path, "w", encoding="utf-8") as f:
                f.write(original.replace(old, new, 1))
            try:
                EX.extract(stem)
                caught = False
            except Exception:                                  # noqa: BLE001
                caught = True
        finally:
            with open(path, "w", encoding="utf-8") as f:
                f.write(original)
        check(f"CAUGHT  {label}", caught)


# ---------------------------------------------------------------------------
# The corpus under test
# ---------------------------------------------------------------------------

def pairs() -> dict:
    out = {}
    for stem in EX.STEMS:
        out[stem] = [GEN.path_for(stem)]
    return out


def _cases(path: str) -> list[dict]:
    with open(path, "rb") as f:
        doc = tomllib.load(f)
    consts = dict(doc.get("constants") or {})

    def sub(text):
        out, rest = [], text
        while "${" in rest:
            i = rest.index("${")
            j = rest.index("}", i)
            out.append(rest[:i])
            out.append(consts.get(rest[i + 2:j], rest[i:j + 1]))
            rest = rest[j + 1:]
        return "".join(out) + rest

    srcs = {sub(k): sub(v) for k, v in (doc.get("source") or {}).items()}
    return [{"name": c["name"], "args": [sub(a) for a in c.get("args", [])],
             "env": dict(c.get("env") or {}), "sources": srcs, "raw": c}
            for c in doc.get("case") or []]


# ---------------------------------------------------------------------------
# SECTION 0 / poisoning
# ---------------------------------------------------------------------------

def _poison_value(value: str) -> str | None:
    """Change the LAST alphanumeric character. CHANGE, never ADD: a poison that
    ADDS a character leaves the original substring intact, and a
    substring-checked claim is then still satisfied -- batch 2 §27's rule,
    inherited."""
    for i in range(len(value) - 1, -1, -1):
        if value[i].isalnum():
            repl = "Z" if value[i] != "Z" else "Q"
            return value[:i] + repl + value[i + 1:]
    return None


def _json_string_leaves(raw: dict) -> list[tuple]:
    """`(dotted path, value)` for every STRING leaf of a case's `json` table."""
    out = []

    def walk(prefix, node):
        if isinstance(node, dict):
            for k, v in node.items():
                walk(f"{prefix}.{k}" if prefix else k, v)
        elif isinstance(node, str) and node:
            out.append((prefix, node))
    walk("", raw.get("json") or {})
    return out


def _poisonable(case: dict):
    """The ONE claim this probe poisons, by a fixed priority:

        1. a `*_contains` needle             (a real substring claim)
        2. a `json_count` needle             (ruling 3's amended clause 4)
        3. a non-empty `json` STRING leaf     (an exact json pin)
        4. a non-empty exact stream pin
        5. an exactly-empty stream or json pin  (nothing to poison inside it)
        6. the exit claim

    Returns `(kind, key, value, poisoned)`. The order puts the substring claims
    first because they are the ones a coverage tool can see, so a case that
    carries one is probed where the audit has its best chance -- which keeps
    arm A's figure a LOWER bound rather than an artificially low one.
    """
    raw = case["raw"]
    for key in ("stdout_contains", "stderr_contains", "stdout_absent",
                "stderr_absent"):
        for v in raw.get(key, []):
            p = _poison_value(v)
            if p:
                return ("needle", key, v, p)
    for claim in raw.get("json_count", []):
        p = _poison_value(claim["needle"])
        if p:
            return ("json_count", "json_count", claim["needle"], p)
    for path, v in _json_string_leaves(raw):
        p = _poison_value(v)
        if p:
            return ("json", "json." + path, v, p)
    for key in ("stdout", "stderr"):
        v = raw.get(key)
        if isinstance(v, str) and v:
            p = _poison_value(v)
            if p:
                return ("exact", key, v, p)
    for key in ("stdout", "stderr"):
        if raw.get(key) == "":
            return ("empty", key, "", None)
    for path, v in _json_string_leaves(raw):
        if v == "":
            return ("empty", "json." + path, "", None)
    return ("exit", "exit", raw.get("exit"), None)


def section0():
    print("\nSECTION 0 -- the probe's own priority is gated, not documented")
    both = {"raw": {"stdout": "", "stderr_contains": ["E5506"], "exit": "failure"}}
    jsonly = {"raw": {"json": {"stdout": "7\n7\n", "stderr": ""},
                      "exit": "success"}}
    counted = {"raw": {"json": {"stderr": ""},
                       "json_count": [{"path": "stdout", "needle": "ok 1",
                                       "at_least": 1}], "exit": "success"}}
    only_exit = {"raw": {"exit": "failure"}}
    check("a case with BOTH an empty pin and a needle is probed on the NEEDLE",
          _poisonable(both)[0] == "needle", repr(_poisonable(both)))
    check("a case whose only substantive claim is a json STRING leaf is probed "
          "on that leaf", _poisonable(jsonly)[0] == "json",
          repr(_poisonable(jsonly)))
    check("a case carrying a json_count is probed on the COUNT needle, not on "
          "the empty json leaf beside it",
          _poisonable(counted)[0] == "json_count", repr(_poisonable(counted)))
    check("a case with only an exit claim is `exit`",
          _poisonable(only_exit)[0] == "exit", repr(_poisonable(only_exit)))


def _rendered_forms(value: str) -> list[str]:
    """Every spelling the GENERATOR could have written this value in, body only.

    Not one spelling: a value with a newline is rendered as a TOML MULTI-LINE
    basic string (real newlines on disk) and one without as an inline string
    (`\\n` escapes). A poison rewrite that knows only the escaped spelling
    silently matches nothing on every exact pin -- which is how batch 4's probe
    first reported arm A firing 0 of 196 times, a perfect blind-spot rate that
    looked like a finding. The forms come from the shared emitter, so the two
    cannot disagree.
    """
    # THE QUOTES ARE PART OF THE FORM, and leaving them off is not a
    # simplification. A short needle rendered inline (`needle = "1"`) has a
    # one-character body, and a bare `1` occurs in `at_least = 1`, in case names
    # and in argv -- so a quote-less search rewrites the wrong token or nothing
    # useful, and six `json_count` poisons rewrote nothing at all. The inline
    # form therefore carries its delimiters; the multi-line form is a body,
    # which is unambiguous by construction because it spans lines.
    forms = [toml_string(value, multiline=False)]
    multi = toml_string(value)
    if multi.startswith('"""'):
        forms.append(multi[len('"""\n'):-len('"""')])
    return [f for f in forms if f]


def _parses(text: str) -> bool:
    try:
        tomllib.loads(text)
        return True
    except tomllib.TOMLDecodeError:
        return False


def _write_poisoned(path, case_name, old, new, all_sites) -> bool:
    """Rewrite one claim (or every occurrence) and REQUIRE the result to parse.

    A value is rendered in one of two spellings and this tries both, but the
    two are not interchangeable in place: substituting a MULTI-LINE body into an
    inline `json.stdout = "..."` produces a file `tomllib` rejects, and a probe
    that ships a syntactically broken case file measures the parser rather than
    the gate. So a candidate rewrite is accepted only if the whole file still
    parses; otherwise the next spelling is tried, and if none works the poison
    is reported as having rewritten nothing -- which section 3 asserts against.
    """
    with open(path, encoding="utf-8") as f:
        text = f.read()
    olds, news = _rendered_forms(old), _rendered_forms(new)
    if all_sites:
        for o, n in zip(olds, news):
            got = text.replace(o, n)
            if got != text and _parses(got):
                with open(path, "w", encoding="utf-8") as f:
                    f.write(got)
                return True
        return False
    blocks = text.split("\n[[case]]\n")
    for i, b in enumerate(blocks):
        if i == 0:
            continue
        if not re.search(r'^name = "%s"$' % re.escape(case_name), b, re.M):
            continue
        for o, n in zip(olds, news):
            if o not in b:
                continue
            trial = list(blocks)
            trial[i] = b.replace(o, n, 1)
            got = "\n[[case]]\n".join(trial)
            if got != text and _parses(got):
                with open(path, "w", encoding="utf-8") as f:
                    f.write(got)
                return True
        return False
    return False


# ---------------------------------------------------------------------------
# The three arms
# ---------------------------------------------------------------------------

_COMPLEMENT: dict[str, str] = {}


def _source_for_audit(stem: str) -> str:
    """The `.rs` arm A is run against -- the migrated complement for a trim.

    Ruling 19: `audit-case-migration.py` is a FORWARD coverage gate, so its
    correct left-hand side for a U4 trim is the migrated complement. Against the
    post-trim file it is red on the control too, and an arm that fires
    unconditionally measures nothing.
    """
    if stem not in EX.PRE_TRIM:
        return os.path.join(TESTS, stem + ".rs")
    if stem in _COMPLEMENT:
        return _COMPLEMENT[stem]
    d = tempfile.mkdtemp(prefix="t19b5-probe-")
    pre = os.path.join(d, "pre.rs")
    with open(pre, "wb") as f:
        f.write(subprocess.run(
            ["git", "show",
             f"{EX.PRE_TRIM[stem]}:crates/kali_cli/tests/{stem}.rs"],
            cwd=REPO, capture_output=True, check=True).stdout)
    comp = os.path.join(d, stem + ".rs")
    r = subprocess.run([sys.executable, os.path.join(PILOT, "migrated_complement.py"),
                        pre, os.path.join(TESTS, stem + ".rs")],
                       cwd=REPO, capture_output=True, text=True, check=True)
    with open(comp, "w", encoding="utf-8") as f:
        f.write(r.stdout)
    _COMPLEMENT[stem] = comp
    return comp


def _audit(stem, tomls) -> int:
    p = subprocess.run(
        [sys.executable, os.path.join(REPO, "scripts/audit-case-migration.py"),
         _source_for_audit(stem)] + list(tomls),
        capture_output=True, text=True, cwd=REPO)
    return p.returncode


def _fidelity_missing(stem, tomls) -> int:
    _sc, _tc, missing, _extra = fidelity.diff([_source_for_audit(stem)],
                                              list(tomls))
    return len(missing)


def _real_binary(case, kind, value) -> bool:
    """Does the REAL binary SATISFY the poisoned claim?

    Arm C fires when it does NOT -- which is exactly what the trial in
    `cargo test --test cases` does. The polarity is spelled out because the
    first version of this function returned "the poisoned value is ABSENT",
    which the caller then negated a second time: arm C read 73/178, exactly the
    count of exit-and-empty cases, and every substring poison silently scored as
    caught-by-nothing. A double negation in a probe arm reports the arm's own
    inverse and looks like a finding about the corpus.
    """
    with tempfile.TemporaryDirectory() as d:
        for k, body in case["sources"].items():
            with open(os.path.join(d, k), "w", encoding="utf-8") as f:
                f.write(body)
        env = dict(os.environ)
        env.update(case["env"])
        r = subprocess.run([KALI] + case["args"], cwd=d, capture_output=True,
                           env=env)
    out = r.stdout.decode("utf-8", "replace")
    err = r.stderr.decode("utf-8", "replace")
    if kind == "exit":
        return True          # an exit poison is not expressible; see section 5
    if kind in ("needle", "json_count", "json", "exact"):
        return value in out or value in err
    return True


# ---------------------------------------------------------------------------
# Sections 2-5
# ---------------------------------------------------------------------------

def run_poisons(label, per_pair):
    corpus = pairs()
    fired = {"A": 0, "B": 0, "C": 0}
    # Arm C's total is TWO populations and printing one number for both reads as
    # 178 measurements when it is not. `C_measured` is a real observation of the
    # binary (the poisoned value is not in its output); `C_construction` is the
    # exit/empty population, which fires unconditionally because such a poison is
    # not expressible as a rewrite at all -- the TRIAL still covers it, but this
    # probe did not measure it here.
    fired_c = {"measured": 0, "construction": 0}
    total = 0
    misses = []
    # M8: the SINGLE-SITE poison's rewrite was unasserted, while the all-sites
    # arm asserted its own. A single-site poison that rewrote nothing lands in
    # `misses` and is then classified by the all-sites re-poison as a
    # declination or a blind spot -- a finding manufactured out of a no-op,
    # which is the same defect class as probe defect #2. Both arms assert now.
    no_rewrite = []
    for stem, tomls in corpus.items():
        base_a = _audit(stem, tomls)
        base_b = _fidelity_missing(stem, tomls)
        if base_a != 0:
            check(f"CONTROL: {stem} audits clean before poisoning", False,
                  f"rc={base_a}")
        for path in tomls:
            with open(path, encoding="utf-8") as f:
                original = f.read()
            cases = _cases(path)
            if per_pair:
                cases = cases[:1]
            for case in cases:
                kind, key, value, poisoned = _poisonable(case)
                total += 1
                rewrote = None
                try:
                    if poisoned is not None:
                        rewrote = _write_poisoned(path, case["name"], value,
                                                  poisoned, False)
                    a = _audit(stem, tomls)
                    b = _fidelity_missing(stem, tomls)
                finally:
                    with open(path, "w", encoding="utf-8") as f:
                        f.write(original)
                if poisoned is not None and not rewrote:
                    no_rewrite.append(f"{stem}::{case['name']} ({kind} {key})")
                c = not _real_binary(case, kind, poisoned or value)
                if a != base_a:
                    fired["A"] += 1
                else:
                    misses.append((stem, path, case, kind, key, value, poisoned))
                if b != base_b:
                    fired["B"] += 1
                if c:
                    fired["C"] += 1
                    fired_c["measured"] += 1
                elif kind in ("exit", "empty"):
                    # an exit or exactly-empty poison is not expressible as a
                    # rewrite; the TRIAL still covers it, which is why the case
                    # carries `exit`/`stdout = ""` at all
                    fired["C"] += 1
                    fired_c["construction"] += 1
    print(f"  {total} case(s) poisoned {label}")
    for arm in "AB":
        print(f"  arm {arm} fired on {fired[arm]:4d}/{total}")
    print(f"  arm C fired on {fired['C']:4d}/{total}"
          f"   ({fired_c['measured']} MEASURED against the real binary, "
          f"{fired_c['construction']} BY CONSTRUCTION -- exit/exactly-empty "
          f"poisons are not expressible as a rewrite and fire unconditionally)")
    check("every single-site poison actually rewrote something",
          not no_rewrite,
          f"{len(no_rewrite)} rewrote nothing: {no_rewrite[:5]}")
    return total, misses


def section3(misses):
    print("\nSECTION 3 -- declination or blind spot: poison EVERY occurrence")
    verdicts = collections.Counter()
    rewrote_nothing = 0
    # THE BLIND SPOTS ARE BROKEN DOWN MECHANICALLY, not by hand. Batch 5's
    # report classified them into three groups by stem and needle and named that
    # split a measurement, but the split appeared nowhere in this tooling -- a
    # hand reading presented as an output. It is computed here, keyed on the
    # pair and the claim kind, so the report can quote the probe instead.
    blind: dict = collections.defaultdict(list)
    for stem, path, case, kind, key, value, poisoned in misses:
        if poisoned is None:
            verdicts["not expressible as a rewrite (exit / exactly-empty pin)"] += 1
            continue
        with open(path, encoding="utf-8") as f:
            original = f.read()
        try:
            changed = _write_poisoned(path, case["name"], value, poisoned, True)
            rc = _audit(stem, [path])
            # WHY it is a blind spot, measured on the poisoned bytes rather than
            # reasoned about afterwards: does the literal STILL OCCUR in the
            # file once every rendered claim carrying it has been rewritten? If
            # it does, the audit is looking at a surviving copy -- aliasing. If
            # it does not, the audit never demanded that literal at all, which
            # is a different mechanism and a different fix.
            with open(path, encoding="utf-8") as f:
                after = f.read()
        finally:
            with open(path, "w", encoding="utf-8") as f:
                f.write(original)
        if not changed:
            rewrote_nothing += 1
            continue
        verdicts["declination" if rc != 0 else "BLIND SPOT"] += 1
        if rc == 0:
            mech = ("ALIASED: the literal survives elsewhere in the case file"
                    if value in after else
                    "NOT DEMANDED: the literal is gone from the case file and "
                    "the audit still passes")
            blind[(stem, kind, key, mech)].append(value)
    for k, v in verdicts.most_common():
        print(f"  {v:6d}  {k}")
    check("every all-sites poison actually rewrote something",
          rewrote_nothing == 0, f"{rewrote_nothing} rewrote nothing")
    if blind:
        print("\n  THE BLIND SPOTS, BROKEN DOWN -- pair, claim kind, mechanism, "
              "and the distinct needles:")
        tot = 0
        by_mech: collections.Counter = collections.Counter()
        for (stem, kind, key, mech), values in sorted(
                blind.items(), key=lambda kv: -len(kv[1])):
            distinct = sorted(set(values))
            shown = ", ".join(repr(v) for v in distinct[:6])
            if len(distinct) > 6:
                shown += f", ... ({len(distinct)} distinct)"
            longest = max(len(v) for v in distinct)
            tot += len(values)
            by_mech[mech.split(":")[0]] += len(values)
            print(f"  {len(values):6d}  {stem}  {kind} ({key})")
            print(f"          mechanism: {mech}")
            print(f"          needles: {shown}")
            print(f"          longest needle: {longest} char(s)")
        print(f"  {tot:6d}  TOTAL blind spots, over {len(blind)} "
              f"(pair, claim kind, mechanism) group(s)")
        for mech, n in by_mech.most_common():
            print(f"  {n:6d}  BY MECHANISM: {mech}")


def section5(misses):
    print("\nSECTION 5 -- every arm-A miss classified")
    counts = collections.Counter()
    for _stem, _path, _case, kind, _key, _value, poisoned in misses:
        if kind == "exit":
            counts["exit: out of the audit's scope by design"] += 1
        elif poisoned is None:
            counts["an exactly-empty pin: nothing to poison inside it"] += 1
        else:
            counts["a literal claim the audit did not see -- see section 3"] += 1
    for k, v in counts.most_common():
        print(f"  {v:6d}  {k}")


def main() -> int:
    if not os.path.exists(KALI):
        print(f"kali binary not found at {KALI}")
        return 2
    section0()
    section1()
    print("\nSECTION 2 -- three arms, one poison per pair")
    run_poisons("(one per pair)", per_pair=True)
    print("\nSECTION 4 -- EXHAUSTIVE: every case poisoned, one at a time")
    total, misses = run_poisons("one at a time", per_pair=False)
    section3(misses)
    section5(misses)
    for d in _COMPLEMENT.values():
        shutil.rmtree(os.path.dirname(d), ignore_errors=True)
    print()
    if FAILURES:
        print(f"PROBE FAILED -- {len(FAILURES)} arm(s):")
        for f in FAILURES:
            print(f"  {f}")
        return 1
    print(f"PROBE OK -- {len(MUTATIONS)} refusal(s); {total} exhaustive poison(s)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
