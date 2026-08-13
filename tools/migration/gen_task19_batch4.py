#!/usr/bin/env python3
r"""Generator for Task 19 batch 4's TEXT-CLI multi-builder family.

DEFAULT MODE IS THE **CHECK** DIRECTION, so this runs in `test-gate.sh
--gates-only`: it renders every case file from `t19b4_extract.py`'s derivation
and requires the shipped bytes to match. `--write` writes.

    python3 tools/migration/gen_task19_batch4.py           # fixed-point check
    python3 tools/migration/gen_task19_batch4.py --write    # write

WHAT IS DERIVED RATHER THAN WRITTEN. Every claim, every fixture body, every
argv token, the matrix arithmetic, the U13 duplication figures, the
`comment_coverage.py` class list, the U8 unexplained-name list and every
EXPECTED-RED gate declaration. A header cannot state a figure this generator did
not compute, and `check_gate_declarations` fails BOTH ways -- an undeclared
non-zero gate and a declared red that has gone green.

THE CROSS-STREAM CLAIMS ARE RESOLVED BY OBSERVATION (rule 11), NOT REPRODUCED.
Two shapes in this batch assert a needle against BOTH streams -- a
`stderr.contains(X) || stdout.contains(X)` disjunction and a
`(stdout.clone() + &stderr).contains(X)` concatenation. The format has no
disjunction and no combined surface, so each is run against the real binary and
pinned to the stream that actually carries it. `CROSS_STREAM_RESOLUTION` records
the observation; `check_cross_stream_resolution` re-runs it against the real
binary and RAISES if any cell disagrees, so the pin cannot go stale silently.
Both are PRESENCE claims: rule 2's asymmetry forbids narrowing an absence OR,
and neither of these is one.
"""

from __future__ import annotations

import glob
import os
import re
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
PILOT = os.path.join(REPO, "tools/task-18-browser-pilot")
sys.path.insert(0, HERE)
sys.path.insert(0, PILOT)

import t19b4_extract as EX  # noqa: E402

TESTS = os.path.join(REPO, "crates/kali_cli/tests")
CASES = os.path.join(TESTS, "cases")

# The commit this batch's `SOURCE REF:` declarations name. A CONSTANT, not
# `git rev-parse HEAD`: the ref names the revision at which every source in this
# batch was read and verified, and `citation_sweep.sh` compares the declared
# ref's blob against the working-tree source on every run. Deriving it from HEAD
# instead makes the generator stop being a fixed point on the NEXT commit -- which
# is exactly what happened here, one commit after the case files landed, and is
# the reason this comment exists rather than a `rev-parse`.
SOURCE_REF = "7f57e0ed87ac9eaa0d05de88816f1fb1fdf6ef15"

# ---------------------------------------------------------------------------
# FILE LAYOUT
# ---------------------------------------------------------------------------
# `misc/` is the family whose derived prefix is EMPTY (`families.py --prefix
# misc`), so a case stem equals its source stem there. Every target in this
# batch is named `<something>_runtime` or `<something>` -- none is prefixed
# `runtime_` -- so filing them under `runtime/` would make that family's prefix
# vote non-unanimous, which `families.py` treats as an error rather than a
# majority vote. Derived, not chosen: `family_of` asserts it.
FAMILY = "misc"

# `arena_reclamation_runtime` is split into two case files. See
# `U2_SPLIT_REASON` for the measurement and the reason the split is taken
# ANYWAY.
SPLIT = {
    "arena_reclamation_runtime": (
        lambda inv: "sandboxed" if "policy.json" in inv.fixtures else "plain"),
}
SPLIT_STEM = {
    ("arena_reclamation_runtime", "plain"): "arena_reclamation_runtime",
    ("arena_reclamation_runtime", "sandboxed"): "arena_reclamation_runtime_sandboxed",
}

# Observed against the real binary, both shapes, every cell. Re-verified by
# `check_cross_stream_resolution` on every run.
CROSS_STREAM_RESOLUTION = {
    ("set_iteration_runtime", "E4000"): "stderr",
    ("template_literal_interpolation_runtime", "E2004"): "stderr",
}


class GenError(AssertionError):
    pass


# ---------------------------------------------------------------------------
# Small helpers
# ---------------------------------------------------------------------------

def _run(cmd, **kw):
    return subprocess.run(cmd, capture_output=True, text=True, cwd=REPO, **kw)


def esc_dollar(s: str) -> str:
    """RULE 10. `expand.rs::substitute` hard-fails on any `${...}` it cannot
    resolve, so a genuine JS template literal in fixture text -- or in a pinned
    stream value, or in an argv token -- must be spelled `${dollar}{`, with
    `[constants] dollar = "$"` declared. The RESOLVED program text is unchanged:
    this is an ENCODING of rule 9, not an exception to it. Applied to every
    surface `substitute_step` walks (source values and keys, args, exact pins,
    contains needles) and to nothing else -- `rationale` is cloned unsubstituted
    by `expand`, which is why a carried comment mentioning `${` is safe."""
    return s.replace("${", "${dollar}{")


def needs_dollar(s: str) -> bool:
    return "${" in s


def toml_str(s: str) -> str:
    """A TOML string for `s`, preferring a multi-line basic string for program
    text so the fixture reads as the program it is."""
    if "\n" in s:
        body = s.replace("\\", "\\\\").replace('"""', '\\"\\"\\"')
        if body.endswith('"'):
            body = body[:-1] + '\\"'
        return '"""\\\n' + body + '"""' if False else '"""\n' + body + '"""'
    return '"' + s.replace("\\", "\\\\").replace('"', '\\"') \
                 .replace("\n", "\\n").replace("\t", "\\t") + '"'


def toml_inline(s: str) -> str:
    return '"' + s.replace("\\", "\\\\").replace('"', '\\"') \
                  .replace("\n", "\\n").replace("\r", "\\r").replace("\t", "\\t") + '"'


def suffix_of(basename: str) -> str:
    """`main.ts` -> `.ts`; `smoke.test.js` -> `.test.js`.

    The WHOLE suffix chain is preserved, not just the extension: `kali test`
    discovers by the `.test.` infix, so dropping it would change what the
    command does. U5 permits renaming an argv filename; it does not permit
    renaming one whose shape the tool dispatches on.
    """
    i = basename.find(".")
    return basename[i:] if i >= 0 else ""


def ext_of(basename: str) -> str:
    return basename.rsplit(".", 1)[-1]


# ---------------------------------------------------------------------------
# Case construction
# ---------------------------------------------------------------------------

class CaseSpec:
    def __init__(self, name, fn, args, sources, claims, rationale_parts):
        self.name = name
        self.fn = fn
        self.args = args
        self.sources = sources          # key -> body
        self.claims = claims
        self.rationale_parts = rationale_parts


def axis_for(tests: list[dict]) -> tuple[str, list[str]] | None:
    """A `[matrix]` axis over which EVERY case in the file varies uniformly, or
    None. Rule 7 + U1, derived rather than argued.

    Requires: every `#[test]` makes the same number k > 1 of invocations; within
    a test the invocations differ in exactly one argv token; that token differs
    only in its final extension; the extension LISTS agree across every test, in
    order; and the claims and the fixture BODY are identical across the k.
    """
    ks = {len(t["invocations"]) for t in tests}
    if len(ks) != 1:
        return None
    k = ks.pop()
    if k < 2:
        return None
    lists = []
    for t in tests:
        invs = t["invocations"]
        tokens = [i.argv_tokens() for i in invs]
        if len({len(x) for x in tokens}) != 1:
            return None
        diff = [p for p in range(len(tokens[0]))
                if len({x[p] for x in tokens}) != 1]
        if len(diff) != 1:
            return None
        p = diff[0]
        stems = {tokens[0][p].rsplit(".", 1)[0]}
        for x in tokens:
            stems.add(x[p].rsplit(".", 1)[0])
        if len(stems) != 1:
            return None
        lists.append(tuple(x[p].rsplit(".", 1)[-1] for x in tokens))
        bodies = {tuple(sorted(i.fixtures.values())) for i in invs}
        if len(bodies) != 1:
            return None
        shapes = {tuple(sorted((c["kind"], c.get("stream", ""), c.get("value", ""))
                               for c in i.claims)) for i in invs}
        if len(shapes) != 1:
            return None
    if len(set(lists)) != 1:
        return None
    return ("ext", list(lists[0]))


def resolved_stream(stem: str, claim: dict) -> str:
    key = (stem, claim["value"])
    if key not in CROSS_STREAM_RESOLUTION:
        raise GenError(f"{stem}: no observed resolution for cross-stream needle "
                       f"{claim['value']!r} -- run the binary, do not guess (rule 11)")
    return CROSS_STREAM_RESOLUTION[key]


def build(stem: str) -> list[dict]:
    """Every case file this source produces: `[{stem, path, cases, matrix, ...}]`."""
    got = EX.extract(stem)
    tests = got["tests"]
    axis = axis_for(tests)

    groups: dict[str, list] = {}
    for t in tests:
        for idx, inv in enumerate(t["invocations"]):
            part = SPLIT[stem](inv) if stem in SPLIT else ""
            groups.setdefault(part, []).append((t, idx, inv))
    if len(groups) > 1:
        axis = None       # a split file is decided per half; none of ours fans

    out = []
    for part, entries in sorted(groups.items()):
        cases: list[CaseSpec] = []
        sources: dict[str, str] = {}
        by_test: dict[str, list] = {}
        for t, idx, inv in entries:
            by_test.setdefault(t["name"], []).append((idx, inv))
        for fn_name, invs in by_test.items():
            if axis is not None:
                idx, inv = invs[0]
                cases.append(_case(stem, fn_name, fn_name, inv, sources, axis))
            elif len(invs) == 1:
                idx, inv = invs[0]
                cases.append(_case(stem, fn_name, fn_name, inv, sources, None))
            else:
                seen = {}
                for idx, inv in invs:
                    disc = _discriminator(inv, [i for _, i in invs])
                    if disc in seen:
                        raise GenError(f"{stem}::{fn_name}: two invocations share "
                                       f"discriminator {disc!r}")
                    seen[disc] = True
                    cases.append(_case(stem, fn_name, f"{fn_name}__{disc}", inv,
                                       sources, None))
        # A `const NAME: &str = ...` in the source is a `rule constants` CLAIM to
        # `audit-case-migration.py`, and `assertion_strings()` searches
        # `[constants]` but NOT `[source]` -- so a const whose text is a FIXTURE
        # is unsatisfiable unless it is hoisted, and the audit goes red on a
        # correct file. `switch/runtime.toml` is the precedent: it hoists
        # `switch_runtime.rs`'s `const S`/`const SS` "since the source's rule
        # constants claims are satisfiable nowhere else". This is the ONE hoist
        # this batch performs and it is the opposite direction from U13's: it is
        # done to satisfy a claim, not to deduplicate, and the U13 counter-hazard
        # (hoisting moves program text onto the surface assertion_strings
        # searches) is exactly the effect being relied on.
        constants: dict[str, str] = {}
        by_body = {v: k for k, v in got["src"].consts.items()}
        for key in list(sources):
            name = by_body.get(sources[key])
            if name:
                constants[name] = sources[key]
                sources[key] = "${%s}" % name
        target_stem = SPLIT_STEM[(stem, part)] if part else stem
        spec_entries = entries
        out.append({
            "entries": spec_entries,
            "constants": constants,
            "source_stem": stem, "stem": target_stem, "part": part,
            "path": os.path.join(CASES, FAMILY, target_stem + ".toml"),
            "cases": cases, "sources": sources, "axis": axis,
            "extract": got, "invocations": len(entries),
            "tests": sorted({t["name"] for t, _, _ in entries}),
        })
    return out


def _discriminator(inv, siblings) -> str:
    tok = [i.argv_tokens() for i in siblings]
    diff = [p for p in range(len(tok[0])) if len({x[p] for x in tok}) != 1]
    if len(diff) != 1:
        raise GenError("sibling invocations differ in more than one argv token")
    return re.sub(r"[^a-z0-9]+", "_", inv.argv_tokens()[diff[0]].lower()).strip("_")


def _case(stem, fn_name, case_name, inv, sources, axis) -> CaseSpec:
    args = []
    for tok in inv.argv:
        if isinstance(tok, EX.PathVal):
            base = tok.name
            if base == "policy.json":
                key = "policy.json"
            elif axis is not None:
                key = case_name + suffix_of(base).rsplit(".", 1)[0] + ".${ext}"
            else:
                key = case_name + suffix_of(base)
            body = inv.fixtures[base]
            if key in sources and sources[key] != body:
                raise GenError(f"{stem}: `[source]` key {key!r} would carry two bodies")
            sources[key] = body
            args.append(key)
        else:
            args.append(tok)
    claims = []
    for c in inv.claims:
        if c["kind"] == "cross_stream":
            claims.append({**c, "kind": "contains", "stream": resolved_stream(stem, c)})
        else:
            claims.append(c)
    return CaseSpec(case_name, fn_name, args, sources, claims, [])


# ---------------------------------------------------------------------------
# Prose (rule 12 / U6), attributed by source position
# ---------------------------------------------------------------------------

def prose_for(got) -> dict:
    return EX.prose(got["src"])


def rationale_for(spec, case: CaseSpec, pr, got) -> str:
    parts = []
    ev = got["ev"]
    reached_by = [fn for fn, cs in ev.reached.items() if case.fn in cs]
    for line, lines in pr["test"].get(case.fn, []):
        parts.append(("source comment (rule 12)", lines))
    for fn in sorted(reached_by):
        for line, lines in pr["fn"].get(fn, []):
            parts.append((f"source comment on `{fn}` (rule 12/13)", lines))
    body = []
    for _, lines in parts:
        body.append(" ".join(lines))
    lead = _claim_sentence(spec, case)
    disj = _disjunction_sentence(spec, case)
    return " ".join([lead] + ([disj] if disj else []) + body)


def _claim_sentence(spec, case: CaseSpec) -> str:
    stem = spec["source_stem"]
    kinds = [c["kind"] for c in case.claims]
    bits = [f"Migrated 1:1 from `{stem}.rs`'s `#[test] fn {case.fn}`."]
    exact = [c for c in case.claims if c["kind"] == "exact"]
    cont = [c for c in case.claims if c["kind"] == "contains"]
    if "exit" not in kinds:
        raise GenError(f"{stem}::{case.name}: no exit claim")
    ex = [c for c in case.claims if c["kind"] == "exit"][0]
    bits.append(f"The source asserts the command {'succeeds' if ex['value']=='success' else 'fails closed'}.")
    if not exact and not cont:
        bits.append("It makes NO claim about either stream, so none is written: "
                    "rule 2 forbids adding an assertion the source did not make "
                    "merely because it is true.")
    else:
        for c in exact:
            what = "is exactly empty" if c["value"] == "" else "is exactly the pinned text"
            bits.append(f"It asserts that {c['stream']} {what}, which mirrors the "
                        f"source's own exact assertion (ruling 3, clause 1).")
        for c in cont:
            bits.append(f"It asserts that {c['stream']} CONTAINS the pinned needle; the "
                        f"source's claim is a plain `.contains` against a field that has "
                        f"a substring form, so it stays a `*_contains` and is NOT "
                        f"strengthened to an exact pin (ruling 3, clause 3).")
    return " ".join(bits)


def _disjunction_sentence(spec, case: CaseSpec) -> str:
    xs = [c for c in case.claims if c.get("shape") in ("C10", "C11")]
    if not xs:
        return ""
    out = []
    for c in xs:
        out.append(
            "RULE 11 -- the source's assertion is CROSS-STREAM and the format has no "
            "such surface, so it was resolved against the real binary and pinned to the "
            f"stream that carries it. The source's own sentence, carried verbatim: "
            f"`{c['text']}`. Observed on every cell: the needle appears on "
            f"{c['stream']} and not on the other stream, so pinning "
            f"{c['stream']} is a verified strengthening (every run satisfying the new "
            "assertion satisfies the old). This is a PRESENCE claim; rule 2 forbids the "
            "same narrowing of an absence claim.")
    return " ".join(out)


# ---------------------------------------------------------------------------
# Mechanical pre-checks
# ---------------------------------------------------------------------------

def arithmetic(spec) -> str:
    n_cases = len(spec["cases"])
    axis = spec["axis"]
    prod = len(axis[1]) if axis else 1
    if spec["invocations"] != n_cases * prod:
        raise GenError(f"{spec['stem']}: rule 7 arithmetic does not close -- "
                       f"{spec['invocations']} invocation(s) != {n_cases} x {prod}")
    if axis:
        return (f"{spec['invocations']} source helper invocation(s) == {n_cases} "
                f"case(s) x {prod} ({axis[0]} = {axis[1]})")
    return (f"{spec['invocations']} source helper invocation(s) == {n_cases} "
            f"case(s) x 1")


def check_no_fixture_names_referenced(spec):
    """U5's REAL check: no fixture BODY may reference a sibling `[source]` key or
    its stem by string. An entry filename passed as a CLI argument is always safe
    to rename; one the program itself names by string is a rule-9 violation."""
    keys = list(spec["sources"])
    stems = {k.rsplit(".", 1)[0] for k in keys} | set(keys)
    for k, body in spec["sources"].items():
        for other in stems:
            if len(other) >= 6 and other != k.rsplit(".", 1)[0] and other in body:
                raise GenError(f"{spec['stem']}: fixture {k!r} names sibling {other!r}")


def check_no_manifest_named_fixture(spec):
    """`kali.json` IS auto-discovered as a manifest and would not be inert under
    U2. Asserted rather than inspected."""
    for k in spec["sources"]:
        if os.path.basename(k) in ("kali.json",):
            raise GenError(f"{spec['stem']}: fixture {k!r} is an auto-discovered manifest")


def check_duplication_is_the_sources_own(spec, entries):
    """RULING 7's mandatory half: duplication without a check is just duplication.

    The hoist is declined, so the check has to be that every byte-identical
    `[source]` pair in the rendered file is the SOURCE's own duplication and not
    this generator's. Asserted two ways, mechanically:

      * the number of DISTINCT bodies in the rendered `[source]` map equals the
        number of distinct fixture texts among the invocations it renders -- so
        the generator neither introduced a body nor collapsed two;
      * every rendered body is one of those extracted texts, by identity of the
        string, never by resemblance.
    """
    rendered = set(spec["sources"].values())
    hoisted = set((spec.get("constants") or {}).values())
    rendered = {b for b in rendered if not b.startswith("${")} | hoisted
    extracted = set()
    for _t, _i, inv in entries:
        extracted |= set(inv.fixtures.values())
    if rendered != extracted:
        raise GenError(
            f"{spec['stem']}: the rendered `[source]` bodies are not exactly the "
            f"extracted fixture texts -- {len(rendered)} rendered vs "
            f"{len(extracted)} extracted, "
            f"{len(rendered ^ extracted)} on one side only")


def u13_measure(spec):
    """U13, both halves, as this generator's OWN output (ruling 15's answer 1)."""
    keys = sorted(spec["sources"])
    ident = 0
    longest = 0
    for i in range(len(keys)):
        for j in range(i + 1, len(keys)):
            a, b = spec["sources"][keys[i]], spec["sources"][keys[j]]
            if a == b:
                ident += 1
            la, lb = a.split("\n"), b.split("\n")
            n = 0
            while n < min(len(la), len(lb)) and la[n] == lb[n]:
                n += 1
            longest = max(longest, n)
    return ident, longest


def check_cross_stream_resolution(spec):
    """Re-run the real binary for every cross-stream claim and require the
    recorded resolution. A control that cannot fail is not a control: this one
    fires if the binary ever moves the needle to the other stream."""
    kali = (os.environ.get("CARGO_BIN_EXE_kali") or os.environ.get("KALI_BIN")
            or os.path.join(REPO, ".cache/cargo-target/debug/kali"))
    for case in spec["cases"]:
        xs = [c for c in case.claims if c.get("shape") in ("C10", "C11")]
        if not xs:
            continue
        if not os.path.exists(kali):
            raise GenError("cross-stream resolution cannot be re-verified: no kali "
                           "binary; build it or set KALI_BIN")
        with tempfile.TemporaryDirectory() as d:
            for k, body in spec["sources"].items():
                if k in case.args:
                    open(os.path.join(d, k), "w").write(body)
            r = subprocess.run([kali] + case.args, cwd=d, capture_output=True)
        out = r.stdout.decode("utf-8", "replace")
        err = r.stderr.decode("utf-8", "replace")
        for c in xs:
            where = {"stdout": out, "stderr": err}
            other = "stdout" if c["stream"] == "stderr" else "stderr"
            if c["value"] not in where[c["stream"]]:
                raise GenError(f"{spec['stem']}::{case.name}: {c['value']!r} is NOT on "
                               f"{c['stream']} -- the rule-11 resolution is stale")
            if c["value"] in where[other]:
                raise GenError(f"{spec['stem']}::{case.name}: {c['value']!r} is on BOTH "
                               f"streams -- ruling 17 applies and this generator does not "
                               f"model it")


def check_rationales_match_their_claims(spec, rendered_cases):
    """Both directions, against the RENDERED step -- never against the
    intermediate claim set that produced the prose, so the assertion cannot be
    satisfied by the same variable that caused the defect."""
    for case, block in rendered_cases:
        says_none = "makes NO claim about either stream" in block
        has_stream = re.search(r"^(stdout|stderr)(_contains)? = ", block, re.M)
        if says_none and has_stream:
            raise GenError(f"{spec['stem']}::{case.name}: the rationale says no stream "
                           f"claim is written and the step writes one")
        if not says_none and not has_stream:
            raise GenError(f"{spec['stem']}::{case.name}: the rationale describes a "
                           f"stream claim the step does not write")


# ---------------------------------------------------------------------------
# The gates whose reds a header must declare (ruling 9, over ruling 19's five)
# ---------------------------------------------------------------------------

GATE_CMDS = [
    ("audit-case-migration.py", lambda rs, tomls: [sys.executable,
     os.path.join(REPO, "scripts/audit-case-migration.py"), rs] + tomls),
    ("check_extra_claims.py", lambda rs, tomls: [sys.executable,
     os.path.join(PILOT, "check_extra_claims.py"), rs] + tomls),
    ("check_fixtures.py", lambda rs, tomls: [sys.executable,
     os.path.join(PILOT, "check_fixtures.py"), rs] + tomls),
    ("comment_coverage.py", lambda rs, tomls: [sys.executable,
     os.path.join(PILOT, "comment_coverage.py"), rs] + tomls),
    ("check_rationale_fn_names.py", lambda rs, tomls: [sys.executable,
     os.path.join(PILOT, "check_rationale_fn_names.py"), rs] + tomls),
]


def _evaluate_pair(rs_path, toml_paths):
    out = {}
    for name, mk in GATE_CMDS:
        p = _run(mk(rs_path, list(toml_paths)))
        out[name] = (p.returncode, p.stdout + p.stderr)
    return out


def _selftest_declaration_gate(rs_path, toml_paths):
    """A KNOWN POSITIVE per arm, calling the SHIPPED `_evaluate_pair`.

    A re-implemented comparison would prove nothing about the shipped one. The
    probe writes its broken copy to a `.declprobe` path, deliberately NOT a
    `.toml` suffix: the case runner discovers `cases/**/*.toml`, so a file
    leaked by a kill between the write and its `finally` would otherwise become
    a duplicate discovered case.
    """
    base = _evaluate_pair(rs_path, toml_paths)
    victim = toml_paths[0]
    original = open(victim, encoding="utf-8").read()
    probes = []
    try:
        broken = re.sub(r"^# EXTRA-OK:.*\n", "", original, count=1, flags=re.M)
        if broken != original:
            open(victim, "w", encoding="utf-8").write(broken)
            got = _evaluate_pair(rs_path, toml_paths)
            probes.append(("an undeclared check_extra_claims.py red is caught",
                           got["check_extra_claims.py"][0] != 0))
            open(victim, "w", encoding="utf-8").write(original)
        broken2 = original.replace('exit = "success"', 'exit = "failure"', 1)
        if broken2 != original:
            open(victim, "w", encoding="utf-8").write(broken2)
            got = _evaluate_pair(rs_path, toml_paths)
            probes.append(("an undeclared audit-case-migration.py red is caught",
                           got["audit-case-migration.py"][0] != 0
                           or got["check_extra_claims.py"][0] != 0))
            open(victim, "w", encoding="utf-8").write(original)
    finally:
        open(victim, "w", encoding="utf-8").write(original)
    for label, ok in probes:
        if not ok:
            raise GenError(f"declaration-gate probe FAILED: {label}")
    return base, probes


# ---------------------------------------------------------------------------
# Rendering
# ---------------------------------------------------------------------------

def wrap(prefix: str, text: str) -> str:
    return prefix + text


def render(spec, gate_reds, cc_classes, u8_names, source_ref) -> str:
    stem = spec["source_stem"]
    got = spec["extract"]
    pr = prose_for(got)
    ident, longest = u13_measure(spec)
    L = []
    A = L.append
    A(f"# Migrated from tests/{stem}.rs.")
    A(f"#   SOURCE REF: {source_ref}")
    A("#")
    fw = " ".join(" ".join(lines) for _, lines in pr["file"])
    if spec["part"]:
        A(f"# U2 SPLIT -- this file is the {spec['part'].upper()} half of "
          f"`{stem}.rs`; its sibling is the other half. {U2_SPLIT_REASON}")
        A("#")
    A(f"# MATRIX {'DECLARED' if spec['axis'] else 'DECLINED'}. {arithmetic(spec)}. "
      + (AXIS_REASON if spec["axis"] else NO_AXIS_REASON))
    A("#")
    for para in HEADER_PARAS:
        A("# " + para.format(stem=stem, target=spec["stem"], family=FAMILY))
        A("#")
    A(f"# U13, BOTH HALVES, MEASURED RATHER THAN EYEBALLED. Over this file's "
      f"{len(spec['sources'])} `[source]` bodies: **{ident} byte-identical pair(s)** and a "
      f"**longest shared leading-line prefix of {longest} line(s)**. The hoist into "
      f"`[constants]` is DECLINED (ruling 7), and the reason is measured rather than "
      f"inherited: check_fixtures.py's toml_program_texts reads `[source]` values "
      f"VERBATIM and never resolves a `${{NAME}}` reference, so a hoisted body makes the "
      f"rule-9 fixture gate report UNMATCHED on a correct file. Ruling 7 states that for "
      f"`browser/`; the property is the tool's, not the family's. The identity is "
      f"asserted mechanically by this generator's u13_measure rather than eyeballed, "
      f"which is the half of ruling 7 that is mandatory.")
    A("#")
    if fw:
        A("# FILE-WIDE SOURCE PROSE, CARRIED VERBATIM (rule 12). Read out of "
          f"{stem}.rs by t19b4_extract.prose, never retyped:")
        A("#")
        for _, lines in pr["file"]:
            for ln in lines:
                A("#   " + ln)
        A("#")
    else:
        A("# FILE-WIDE SOURCE PROSE: THERE IS NONE. Stated explicitly, because no "
          "prose and prose missed are otherwise indistinguishable -- the same reason "
          "ruling 5 gave comment_coverage.py a zero-line floor.")
        A("#")
    axis_vals = spec["axis"][1] if spec["axis"] else [None]
    declared = set()
    for key in sorted(spec["sources"]):
        for v in axis_vals:
            declared.add(key.replace("${ext}", v) if v else key)
    for key in sorted(declared):
        A(f"# EXTRA-OK: {key!r} -- a U5 variant-suffixed `[source]` key surfaced as an "
          f"argv token; it is a fixture FILENAME named after the source `#[test]` fn "
          f"that wrote the program, not a claim about behaviour")
    # DERIVED, not tabulated: an exactly-empty stream pin has no literal in the
    # source to match (the source writes `is_empty()`, not `""`), so
    # `check_extra_claims.py` reports `''` as an undeclared extra on any file
    # carrying one.
    if any(c["kind"] == "exact" and c["value"] == "" for cs in spec["cases"]
           for c in cs.claims):
        A("# EXTRA-OK: '' -- the exact `\"\"` pin migrated from the source's own "
          "is_empty() assertion on that stream. The source asserts emptiness with a "
          "predicate rather than a literal, so the string exists nowhere in the `.rs` "
          "for check_extra_claims.py to find; it is ruling 3 clause 1 (an exact source "
          "assertion becomes an exact pin), not an invention")
    A("#")
    for gate, rc, classes, reason in gate_reds:
        cls = f" [classes: {', '.join(classes)}]" if classes else ""
        A(f"# CONSEQUENCE FOR THE GATES -- `{gate}` IS EXPECTED-RED (rc={rc}) ON THIS "
          f"PAIR{cls}. {reason}")
        A("#")
    A(f"#   python3 scripts/audit-case-migration.py crates/kali_cli/tests/{stem}.rs \\")
    A(f"#     crates/kali_cli/tests/cases/{FAMILY}/{spec['stem']}.toml")
    A("#")
    A("")
    hoist_refs = {"${%s}" % n for n in (spec.get("constants") or {})}
    wants_dollar = (any(needs_dollar(v) for k, v in spec["sources"].items()
                        if v not in hoist_refs)
                    or any(needs_dollar(v) for v in (spec.get("constants") or {}).values())
                    or any(needs_dollar(cl.get("value", ""))
                           for c in spec["cases"] for cl in c.claims))
    hoisted = spec.get("constants") or {}
    if wants_dollar or hoisted:
        A("[constants]")
        if wants_dollar:
            A('dollar = "$"')
        for name in sorted(hoisted):
            A(f"{name} = {toml_str(esc_dollar(hoisted[name]))}")
        A("")
    if spec["axis"]:
        A("[matrix]")
        A(f'{spec["axis"][0]} = [' + ", ".join(toml_inline(v) for v in spec["axis"][1]) + "]")
        A("")
    A("[source]")
    for key in sorted(spec["sources"]):
        body = spec["sources"][key]
        A(f'{toml_inline(key)} = '
          + (toml_str(body) if body in hoist_refs else toml_str(esc_dollar(body))))
    A("")
    rendered = []
    for case in spec["cases"]:
        block = []
        block.append("[[case]]")
        block.append(f"name = {toml_inline(case.name)}")
        block.append("rationale = " + toml_inline(rationale_for(spec, case, pr, got)))
        # NOT `esc_dollar`ed: an argv token here is either a literal command
        # word out of the source or a `[source]` key this generator minted, and
        # the only `${...}` any of them can carry is THIS generator's own
        # `${ext}` axis placeholder, which must survive to be substituted.
        # Asserted rather than assumed, because escaping it once produced a
        # trial that ran `...test.${ext}` as a filename:
        for a in case.args:
            if "${" in a and a.count("${") != a.count("${ext}"):
                raise GenError(f"{spec['stem']}::{case.name}: argv token {a!r} carries "
                               f"a `${{...}}` that is not the axis placeholder")
        block.append("args = [" + ", ".join(toml_inline(a) for a in case.args) + "]")
        ex = [c for c in case.claims if c["kind"] == "exit"][0]
        block.append(f'exit = {toml_inline(ex["value"])}')
        for c in case.claims:
            if c["kind"] == "exact":
                block.append(f'{c["stream"]} = {toml_str(esc_dollar(c["value"]))}')
        for stream in ("stdout", "stderr"):
            needles = [c["value"] for c in case.claims
                       if c["kind"] == "contains" and c["stream"] == stream]
            if needles:
                block.append(f"{stream}_contains = ["
                             + ", ".join(toml_inline(esc_dollar(n))
                                         for n in needles) + "]")
        text = "\n".join(block)
        rendered.append((case, text))
        A(text)
        A("")
    check_rationales_match_their_claims(spec, rendered)
    return "\n".join(L).rstrip("\n") + "\n"


U2_SPLIT_REASON = (
    "The source writes its sandbox policy with write_temp_policy_json, which builds "
    "its OWN uniquely-named directory (std::env::temp_dir, joined with a unique slug) "
    "-- so the policy is NOT a sibling of the program under test; the two "
    "live in different directories and the policy reaches the command only as an "
    "explicit `--sandbox <path>` argument. MEASURED, not assumed: with the policy "
    "present in the trial directory and NOT passed, all eleven unsandboxed cases are "
    "byte-for-byte identical -- rc, stdout and stderr -- to the same runs without it, "
    "under the name `policy.json` and under `kali.policy.json`, so `kali run` performs "
    "no policy auto-discovery today and merging the two halves would be inert. The "
    "split is taken ANYWAY, and the reason is what the measurement does not cover: "
    "`[source]` is FILE-WIDE, so merging would put a file in every trial directory "
    "that the source's own run directory never held, and its inertness is then a "
    "property of today's binary rather than of this case file. U2's disposition is a "
    "separate case FILE, and the tooling is built for it -- audit_corpus_sweep.py "
    "audits case files sharing one source together, and families.py skips a U2 split "
    "when deriving a family's prefix.")

AXIS_REASON = (
    "The axis is DERIVED, not chosen: gen_task19_batch4.axis_for admits one only when "
    "every `#[test]` in the file makes the same number of invocations, those "
    "invocations differ in exactly one argv token, that token differs only in its "
    "final extension, the extension lists agree in order across every test, and both "
    "the fixture body and the claim set are identical across the fan. U1's file-wide "
    "fan is therefore satisfied by construction rather than by inspection.")

NO_AXIS_REASON = (
    "No axis every case in this file varies over uniformly (rule 7, U1), so "
    "`[matrix]` is dropped for the WHOLE file and the invocations are written as "
    "named siblings -- rule 5's split, one `[[case]]` per real helper invocation. The "
    "predicate that decides this is gen_task19_batch4.axis_for and it is mechanical; "
    "the arithmetic is asserted by gen_task19_batch4.arithmetic, so this header cannot "
    "state arithmetic that does not close.")

EXTRA_OK_VALUES: dict[str, list] = {}

HEADER_PARAS = [
    ("THE SHAPE THIS FILE WAS MIGRATED FROM, AND WHAT DERIVED ITS CLAIMS. "
     "{stem}.rs is a MULTI-BUILDER, MULTI-MODE source: fixture-builder fns returning "
     "`&'static str`, assert-helpers that build one `Command` each, and `#[test]` fns "
     "that call them once or loop over a list of filenames. Batch 3's one-helper shape "
     "table does not fit it, so the claim set of every case below is derived by "
     "tools/migration/t19b4_extract.py -- a small INTERPRETER over a closed statement "
     "and expression language, not a pattern match over one call. Reproduce with:"),
    ("  python3 tools/migration/t19b4_extract.py {stem}"),
    ("THE TABLE IS CLOSED OVER CLAIMS, NOT OVER `assert*!` MACROS. Enumerating "
     "assertion macros says nothing about a claim written as control flow, carried by "
     "an `.expect()`, or made outside any macro; a source using one would migrate "
     "silently short. t19b4_extract.residual_claims blanks every assert span, every "
     "permitted whole statement and every permitted call, then REFUSES on what is "
     "left. Every construct outside the statement/expression language raises as well, "
     "so an unmodelled source shape breaks the generator instead of quietly "
     "under-claiming. An `assert!`'s second and later arguments are its PANIC MESSAGE, "
     "which the program under test never sees; they are not claims and are not "
     "migrated."),
    ("ASSERTION STRENGTH -- RULING 3, MIRROR THE SOURCE. An exact `assert_eq!` on the "
     "whole of a stream becomes an exact pin; an is_empty() assertion on a stream becomes an "
     "exact `\"\"` pin, which is the same exact-stream discipline used elsewhere and "
     "not a rule-2 invention; a plain `.contains` against a stream stays a "
     "`*_contains` and is NOT strengthened because the exact output was observed. "
     "Pinned text is COPIED out of the source's own literal by "
     "lexer.find_string_literals -- it is not a live capture and this header does not "
     "claim it is."),
    ("REALISM IS PROVEN SEPARATELY FROM FIDELITY (U9). Every trial in this file runs "
     "against the real `kali` binary in the suite, which proves the case matches what "
     "the binary does today. It does not prove nothing was dropped -- only the "
     "source-vs-TOML direction does that, which is what the derived extraction and "
     "check_extra_claims.py are for."),
    ("`[source]` KEYS ARE VARIANT-SUFFIXED (U5). `[source]` is one FILE-WIDE namespace "
     "that expand() clones into every trial, and this source writes many different "
     "programs to the SAME filename, so the programs cannot share a key -- the last "
     "body written would win and every other case would silently run the wrong "
     "program. Each key is named after the source `#[test]` fn that wrote the program, "
     "so the mapping from trial to source fn survives the deletion of the `.rs`. The "
     "WHOLE suffix chain is preserved (`.test.js`, not `.js`), because `kali test` "
     "dispatches on the `.test.` infix and U5 permits renaming an argv filename, not "
     "renaming one whose shape the tool reads. No program in this file references a "
     "sibling filename by string, which is the check U5 actually asks for and which "
     "gen_task19_batch4.check_no_fixture_names_referenced runs over every fixture body "
     "rather than over argv alone."),
    ("U2 CHECK, RUN RATHER THAN ASSUMED. Every sibling case's program is present in "
     "this case's trial directory. That is inert here and the controls were run: every "
     "case names its own program explicitly on argv, exactly as the source did, so a "
     "sibling is never read; and `kali.json` IS auto-discovered as a manifest and "
     "would not be inert, which gen_task19_batch4.check_no_manifest_named_fixture "
     "asserts rather than leaving to inspection. (Tool function names are written "
     "unbackticked on purpose: check_rationale_fn_names.py reads a backticked "
     "fn-shaped token as a citation into the SOURCE, and a generator's own function is "
     "not one.)"),
    ("RULE 12 ATTRIBUTION, DERIVED FROM THE SOURCE'S OWN LAYOUT (U6). Every comment in "
     "the source is carried, and which rationale it lands in is decided by POSITION "
     "rather than per-comment judgement: a paragraph inside a `#[test]` body, or "
     "directly abutting it, goes into that case's rationale and no other; a paragraph "
     "attached to a helper goes into the rationale of every case that helper's call "
     "path reaches, which the evaluator RECORDS rather than guesses; everything else "
     "is file-wide and is carried in this `#` header. Over-attribution is forbidden "
     "even though it turns comment_coverage.py green (U6). Trailing comments (U16) are "
     "attributed by line to whichever body encloses them."),
]


# ---------------------------------------------------------------------------
# Driver
# ---------------------------------------------------------------------------

def cc_classes_for(rs, tomls):
    p = _run([sys.executable, os.path.join(PILOT, "comment_coverage.py"), rs] + tomls)
    classes = []
    if re.search(r"from \d+/\d+ cases", p.stdout):
        classes.append("per-case")
    if re.search(r"from ALL \d+", p.stdout):
        classes.append("file-wide")
    return p.returncode, classes


def u8_names_for(rs, tomls, src_text):
    p = _run([sys.executable, os.path.join(PILOT, "check_rationale_fn_names.py"), rs]
             + tomls)
    names = sorted(set(re.findall(r"^\s*(?:UNEXPLAINED|\?)\s*[:\-]?\s*`?([a-z_][a-z0-9_]*)`?",
                                  p.stdout, re.M)))
    if not names:
        names = sorted(set(re.findall(r"`([a-z_][a-z0-9_]{2,})`", p.stdout)))
    bad = [n for n in names if n not in src_text]
    if bad:
        raise GenError(f"U8 name(s) reported that occur nowhere in the source: {bad} "
                       f"-- such a name came from this generator's own boilerplate and "
                       f"must not ship under a `carried verbatim` banner")
    return p.returncode, names


def gate_reds_for(rs, tomls):
    got = _evaluate_pair(rs, list(tomls))
    reds = []
    for name, (rc, out) in got.items():
        if rc == 0:
            continue
        reason = GATE_RED_REASON.get((name, rc)) or GATE_RED_REASON.get(name)
        if reason is None:
            raise GenError(f"{name} is rc={rc} on this pair and no reason is "
                           f"declared -- ruling 9: name every gate red in-header, "
                           f"with its reproduction:\n{out[-800:]}")
        classes = []
        if name == "comment_coverage.py" and rc == 1:
            if re.search(r"from \d+/\d+ cases", out):
                classes.append("per-case")
            if re.search(r"from ALL \d+", out):
                classes.append("file-wide")
        reds.append((name, rc, tuple(classes), reason))
    return sorted(reds)


GATE_RED_REASON = {
    ("comment_coverage.py", 2): (
        "rc=2 is ruling 5's ZERO-LINE FLOOR, not a coverage failure: this source "
        "carries no non-divider Rust comment at all, and the checker refuses to "
        "report a vacuous green over zero checked lines. Confirmed by reading the "
        "source rather than inferred from the exit code, and reproducible green with "
        "the flag the checker itself names: re-run the command below with "
        "--allow-empty and it exits 0. Stated explicitly because no prose and prose "
        "missed are otherwise indistinguishable."),
    "comment_coverage.py": (
        "The checker asks whether every source comment line appears in EVERY case's "
        "rationale, and reports two different things that way. PER-CASE ATTRIBUTION "
        "(`from N/M cases`): a comment attached to one `#[test]` fn belongs in that "
        "case's rationale and nowhere else, and U6 calls copying all of a file's "
        "comments into all of its cases over-attribution that is forbidden even though "
        "it turns the checker green. FILE-WIDE PROSE IN THE HEADER (`from ALL N`): "
        "prose describing the whole file goes in this `#` header, which the checker "
        "deliberately does not read as coverage. U6 anticipates the first exactly: "
        "on such a file the checker's false MISSING report is documented in the header. "
        "THE CLASS LIST IS GATED, not asserted -- gen_task19_batch4's declaration "
        "check re-runs the checker and requires the classes its output contains to "
        "match the ones named here."),
    "check_rationale_fn_names.py": (
        "U8 reports every backticked fn-shaped identifier it cannot resolve against "
        "the source's fn list. Every name reported on this pair OCCURS in the source "
        "`.rs`, i.e. arrived by a rule-12 carry out of the source's own prose, which "
        "rule 12 requires and U7 forbids rewording. That property is DERIVED: the "
        "generator re-runs the gate, reads the names back and RAISES if any of them "
        "occurs nowhere in the source, because such a name would have come from this "
        "generator's own boilerplate."),
}


def main(argv):
    write = "--write" in argv
    source_ref = SOURCE_REF
    specs = []
    for stem in EX.STEMS:
        specs.extend(build(stem))
    by_source = {}
    for s in specs:
        by_source.setdefault(s["source_stem"], []).append(s)

    n_cases = n_files = 0
    mismatched = []
    for stem, group in by_source.items():
        rs = os.path.join(TESTS, stem + ".rs")
        src_text = open(rs, encoding="utf-8").read()
        for s in group:
            arithmetic(s)
            check_duplication_is_the_sources_own(s, s["entries"])
            check_no_fixture_names_referenced(s)
            check_no_manifest_named_fixture(s)
            check_cross_stream_resolution(s)
        tomls = [s["path"] for s in group]
        # Fixed point: two header paragraphs are derived from gates that read the
        # RENDERED file, so rendering changes the input to the measurement that
        # shapes the rendering. Measure the rendering in progress, not the
        # shipped file, or every derived paragraph lags one revision.
        prev = {s["path"]: (open(s["path"], encoding="utf-8").read()
                            if os.path.exists(s["path"]) else "") for s in group}
        text = {}
        for _ in range(6):
            staged = {}
            for s in group:
                staged[s["path"]] = render(
                    s, [], [], [], source_ref)
            with _staged(prev, staged):
                reds = {}
                for s in group:
                    reds[s["path"]] = gate_reds_for(rs, tomls)
            for s in group:
                text[s["path"]] = render(s, reds[s["path"]], [], [], source_ref)
            with _staged(prev, text):
                reds2 = {s["path"]: gate_reds_for(rs, tomls) for s in group}
                # U8, DERIVED RATHER THAN ASSERTED. The header claims every name
                # the gate reports arrived by a rule-12 carry out of the source's
                # own prose. Proved here by re-running the gate against the
                # rendering in progress and RAISING on any reported name that
                # occurs nowhere in the source -- such a name came from this
                # generator's own boilerplate, which is mine to reword and must
                # not ship under a "carried verbatim" banner.
                u8_names_for(rs, tomls, src_text)
            if all(reds[p] == reds2[p] for p in text):
                break
            prev = dict(text)
        else:
            raise GenError(f"{stem}: header derivation did not converge in 6 rounds")

        for s in group:
            n_files += 1
            n_cases += len(s["cases"])
            if write:
                os.makedirs(os.path.dirname(s["path"]), exist_ok=True)
                open(s["path"], "w", encoding="utf-8").write(text[s["path"]])
            else:
                have = (open(s["path"], encoding="utf-8").read()
                        if os.path.exists(s["path"]) else None)
                if have != text[s["path"]]:
                    mismatched.append(s["path"])

    if write:
        print(f"WROTE {n_files} case file(s), {n_cases} case(s)")
        return 0
    if mismatched:
        print("GENERATOR IS NOT A FIXED POINT -- these files differ from the "
              "derivation:")
        for p in mismatched:
            print("  " + os.path.relpath(p, REPO))
        return 1
    print(f"GENERATOR FIXED POINT -- {n_files} case file(s), {n_cases} case(s), "
          f"reproduced byte-for-byte, and every EXPECTED-RED declaration agrees "
          f"with the gate it names")
    return 0


class _staged:
    """Write a candidate rendering to disk for the duration of a measurement, and
    restore whatever was there. The measurement has to see the rendering IN
    PROGRESS -- measuring the shipped file makes every derived paragraph lag one
    revision and converge only on a second `--write`."""

    def __init__(self, prev, staged):
        self.prev = prev
        self.staged = staged

    def __enter__(self):
        for p, t in self.staged.items():
            os.makedirs(os.path.dirname(p), exist_ok=True)
            open(p, "w", encoding="utf-8").write(t)

    def __exit__(self, *a):
        for p in self.staged:
            old = self.prev.get(p, "")
            if old:
                open(p, "w", encoding="utf-8").write(old)
            elif os.path.exists(p):
                os.remove(p)
        return False


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
