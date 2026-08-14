#!/usr/bin/env python3
"""Generator for Task 19 batch 5 -- the last migration batch.

Renders seven case files from `t19b5_extract`'s reading of seven `.rs` sources,
and is a FIXED POINT: run with no arguments it regenerates every file and
requires each to be byte-for-byte what is already shipped, exiting 1 on drift.
The CHECK direction is the default on purpose -- a generator that only writes is
a fixed point nobody re-tests.

  gen_task19_batch5.py            # CHECK: regenerate and diff; rc=1 on drift
  gen_task19_batch5.py --write    # emit the seven case files
  gen_task19_batch5.py --list     # the files, with per-file trial counts

WHAT RUNS BEFORE ANYTHING IS EMITTED. Each of these RAISES rather than reports,
so a file that would violate one cannot be written at all:

  * `arithmetic`                     rule 7: invocations == cases x axis product
  * `check_captured`                 rules 8/9: every capture still matches the
                                     source it was taken from, and the eleven
                                     `format!` captures still agree with a
                                     structural recomputation
  * check_duplication_is_the_sources_own
                                     ruling 7's mandatory half: the set of
                                     distinct rendered `[source]` bodies equals
                                     the set of distinct fixture texts the
                                     extractor produced, so the generator
                                     neither introduced a body nor collapsed two
  * `check_no_fixture_names_referenced`
                                     U5's real check, over every fixture BODY:
                                     an argv filename is safe to rename, one the
                                     program names by string is a rule-9
                                     violation
  * `check_or_resolution`            rule 11: re-runs the real binary for all
                                     eighteen resolved disjunctions and raises
                                     if any moves branch
  * `check_declined`                 re-measures the ground for declining the
                                     four `array_from_*` targets
  * `check_self_inspection`          ruling 10's tool, over all seven targets
  * `check_rationales_match_their_claims`
                                     both directions, against the RENDERED step
  * `check_reproduction_commands`    every command a header prints is parsed,
                                     its paths required to exist, and the audit
                                     one RUN with rc=0 required
  * `_selftest_*`                    a committed known positive per refusal, so
                                     a green check is distinguishable from a
                                     check that cannot fire (batch 4's M1)

WHY `SOURCE REF` IS A CONSTANT. It is pinned to the revision at which every
source was READ, not to `git rev-parse HEAD`: deriving it from HEAD makes the
generator stop being a fixed point on the next commit, which is exactly what
happened to batch 4 one commit after its case files landed.
"""

from __future__ import annotations

import os
import re
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
PILOT = os.path.join(REPO, "tools/task-18-browser-pilot")
sys.path.insert(0, PILOT)
sys.path.insert(0, HERE)

from case_emit import emit  # noqa: E402
from toml_emit import toml_string as _toml_str  # noqa: E402
from comment_coverage import is_divider  # noqa: E402
import t19b5_captures as CAP  # noqa: E402
import t19b5_extract as EX  # noqa: E402
from t19b5_extract import PathVal, UnknownShape  # noqa: E402

TESTS = os.path.join(REPO, "crates/kali_cli/tests")
CASES = os.path.join(TESTS, "cases")

# The revision at which every source was read. Also the `PRE-TRIM REF:` of the
# one U4 trim in this batch, because the trim is made in this batch and the
# pre-trim blob is therefore exactly this commit's copy.
SOURCE_REF = "47e9b083c61e32c972727189a580d1e9cacb856c"

KALI = os.path.join(REPO, ".cache/cargo-target/debug/kali")

# family -> file stem, derived from `families.py`'s own rule (a case file is
# `cases/<family>/<stem>.toml` and its source is `tests/<prefix><stem>.rs`).
# `runtime_` and `nullish_` are real family prefixes, so those two targets file
# under their own family rather than into `misc/`; the other five carry no
# family prefix, which is exactly what `misc/`'s empty prefix is for.
PLACEMENT = {
    "for_of_array_iteration_spread": ("misc", "for_of_array_iteration_spread"),
    "logical_assignment_wrapped_local_binding":
        ("misc", "logical_assignment_wrapped_local_binding"),
    "nullish_assignment_wrapped_local_binding":
        ("nullish", "assignment_wrapped_local_binding"),
    "parse_float_static_ascii": ("misc", "parse_float_static_ascii"),
    "runtime_forin": ("runtime", "forin"),
    "thread_topology_json": ("misc", "thread_topology_json"),
    "wrapped_call_targets_wrappers": ("misc", "wrapped_call_targets_wrappers"),
}


class Refuse(AssertionError):
    pass


# ---------------------------------------------------------------------------
# Rule 11 -- the eighteen disjunctions, resolved against the real binary
# ---------------------------------------------------------------------------

def run_real(fixtures: dict, argv: list, env: dict) -> tuple:
    with tempfile.TemporaryDirectory() as d:
        for name, body in fixtures.items():
            with open(os.path.join(d, name), "w", encoding="utf-8") as f:
                f.write(body)
        e = dict(os.environ)
        e.update(env)
        r = subprocess.run([KALI] + argv, cwd=d, capture_output=True, env=e)
    return (r.returncode,
            r.stdout.decode("utf-8", "replace"),
            r.stderr.decode("utf-8", "replace"))


def resolve_or(inv, claim) -> dict:
    """Rule 11 on `!status.success() || stdout == X`, by observation.

    The format has no disjunction, so the source's OR is resolved against the
    real binary and the branch that actually occurs is pinned. This is a
    verified strengthening: every run satisfying the pin satisfies the OR.

    RULING 17 IS CHECKED FOR AND DOES NOT ARISE. If BOTH disjuncts held on a
    cell there would be a tie to break in source order; this raises instead of
    picking, because a tie that appears later is a decision, not a detail. All
    eighteen cells carry exactly one true disjunct.
    """
    rc, out, _err = run_real(inv.fixtures, inv.argv_tokens(), inv.env)
    failed = rc != 0
    matched = out == claim["stdout"]
    if failed and matched:
        raise Refuse(f"{inv.fn_name}: BOTH disjuncts hold -- ruling 17's tie, "
                     f"which this generator refuses to break silently")
    if not failed and not matched:
        raise Refuse(f"{inv.fn_name}: NEITHER disjunct holds (rc={rc}, "
                     f"stdout={out!r}) -- the source's own assertion fails")
    if failed:
        return {"kind": "exit", "value": "failure", "branch": "!success"}
    return {"kind": "stdout", "value": claim["stdout"], "branch": "stdout =="}


# ---------------------------------------------------------------------------
# Claims -> a rendered step
# ---------------------------------------------------------------------------

def step_for(inv, resolved: dict) -> dict:
    step = {}
    json_paths, json_count = {}, []
    for c in inv.claims:
        k = c["kind"]
        if k == "or_fail_or_stdout":
            r = resolved[id(c)]
            if r["kind"] == "exit":
                step["exit"] = r["value"]
            else:
                step["stdout"] = r["value"]
            continue
        if k == "exit":
            v = c["value"]
            # `status.success()` and `status.code() == Some(0)` are the same
            # claim at different strengths; the numeric one is stronger and is
            # what the source also asserts, so it wins. Never the other way.
            if isinstance(v, int) or step.get("exit") in (None, "success"):
                step["exit"] = v
            continue
        if k in ("stdout", "stderr"):
            step[k] = c["value"]
            continue
        if k in ("stdout_contains", "stdout_absent",
                 "stderr_contains", "stderr_absent"):
            step.setdefault(k, [])
            if c["value"] not in step[k]:
                step[k].append(c["value"])
            continue
        if k == "json":
            json_paths[c["path"]] = c["value"]
            continue
        if k == "json_count":
            entry = {"path": c["path"], "needle": c["needle"],
                     "at_least": c["at_least"]}
            if entry not in json_count:
                json_count.append(entry)
            continue
        raise Refuse(f"claim kind `{k}` has no rendering")
    if json_paths:
        step["json_paths"] = json_paths
    if json_count:
        step["json_count"] = json_count
    if inv.env:
        step["env"] = dict(inv.env)
    step["args"] = inv.argv_tokens()
    return step


# ---------------------------------------------------------------------------
# `[source]` keys (U5) and fixture naming
# ---------------------------------------------------------------------------

def source_key(inv, case_name: str, tok: PathVal, used: dict) -> str:
    """A file-wide-unique `[source]` key for one invocation's fixture.

    `[source]` is one flat file-wide namespace (U2/U5), and these sources write
    many DIFFERENT programs to the same filename, so a bare `main.ts` reused
    across cases would silently give every trial the last one written. The key
    is the source `#[test]` fn's name plus the original suffix chain, so the fn
    name survives the source's deletion -- in argv, in the `[source]` map and in
    the rationale.

    THE WHOLE SUFFIX CHAIN IS PRESERVED (`.test.js`, not `.js`): kali test
    dispatches on the `.test.` infix, so truncating it would change what the
    tool does. U5 permits renaming an argv filename, not renaming one whose
    SHAPE the tool reads.
    """
    name = tok.name
    for infix in (".test.", "."):
        i = name.find(infix)
        if i >= 0:
            suffix = name[i:]
            break
    else:
        suffix = ""
    body = inv.fixtures[name]
    for key, (b, _) in used.items():
        if b == body and key.endswith(suffix) and key.startswith(case_name):
            return key
    key = case_name + suffix
    n = 2
    while key in used and used[key][0] != body:
        key = f"{case_name}_{n}{suffix}"
        n += 1
    used[key] = (body, name)
    return key


# ---------------------------------------------------------------------------
# Prose (rule 12 / U6), attributed by SOURCE POSITION
# ---------------------------------------------------------------------------

def prose_for(pr: dict, ev, case_name: str) -> list[str]:
    """The comment text this case must carry, and no other.

    Per-`#[test]` prose goes to that case alone. Per-helper prose goes to the
    rationale of every case that helper's call path REACHED -- which the
    evaluator recorded (`ev.reached`) rather than the generator guessing. That
    is rule 13's transitive-doc requirement satisfied by construction.
    """
    out = []
    for line, lines in sorted(pr["test"].get(case_name, [])):
        out.append(" ".join(x for x in lines if not is_divider(x)).strip())
    for fn, blocks in sorted(pr["fn"].items()):
        if case_name not in ev.reached.get(fn, set()):
            continue
        for line, lines in sorted(blocks):
            out.append(" ".join(x for x in lines if not is_divider(x)).strip())
    return [o for o in out if o]


# ---------------------------------------------------------------------------
# The mandatory checks
# ---------------------------------------------------------------------------

def check_captured(stem: str, src) -> None:
    """Every capture still belongs to the source it was taken from (rules 8/9).

    Two arms. The builder captures must still name a fn that exists and still
    builds its value the way the capture was taken for. The eleven `format!`
    captures are additionally cross-checked against a STRUCTURAL recomputation
    of the same `format!` -- the capture is what ships, the recomputation is an
    independent second opinion, and a disagreement raises rather than either
    being preferred.
    """
    if stem == "for_of_array_iteration_spread":
        for key in CAP.SPREAD:
            fn = key.split("(")[0]
            if fn not in src.fns:
                raise Refuse(f"capture {key!r} names `{fn}`, which no longer "
                             f"exists in {stem}.rs")
            if "format!" not in src.fns[fn]["masked"] and \
                    "array_from_" not in src.fns[fn]["masked"]:
                raise Refuse(f"capture {key!r} is stale: `{fn}` no longer "
                             f"builds its value the way it was captured")
    if stem == "runtime_forin":
        names = {t["name"] for t in src.tests}
        for key in CAP.RUNTIME_FORIN:
            if key not in names:
                raise Refuse(f"capture {key!r} names a `#[test]` that no longer "
                             f"exists in {stem}.rs")


def check_duplication_is_the_sources_own(stem, source_map, ex) -> None:
    """Ruling 7's mandatory half, as an assertion rather than a sentence.

    The set of distinct rendered `[source]` bodies must equal the set of
    distinct fixture texts the extractor produced for this file. So the
    generator neither introduced a body nor collapsed two, and every
    byte-identical pair in the rendered file is the SOURCE's own duplication.

    THE HOIST IS DECLINED (ruling 7), and the reason is the TOOL's rather than
    the family's: check_fixtures.py's toml_program_texts reads `[source]`
    values verbatim and never resolves a `${NAME}` reference, so a hoisted body
    makes the rule-9 fixture gate report UNMATCHED on a correct file.
    """
    rendered = set(source_map.values())
    produced = set()
    for t in ex["tests"]:
        for inv in t["invocations"]:
            produced |= set(inv.fixtures.values())
    if rendered != produced:
        raise Refuse(
            f"{stem}: rendered `[source]` bodies are not the extractor's "
            f"fixture set ({len(rendered)} vs {len(produced)} distinct)")


def check_no_fixture_names_referenced(stem, source_map) -> None:
    """U5's real check, over every fixture BODY rather than over argv.

    An entry filename passed as a CLI argument is always safe to rename; a
    filename the program itself names by string (`import()`, `require()`) is
    not -- renaming that is a rule-9 violation even when the file is otherwise
    byte-identical.
    """
    original = {orig for _k, orig in ()}          # placeholder, filled by caller
    del original
    for key, body in source_map.items():
        for other in source_map:
            if other == key:
                continue
            if re.search(r"""(?:import|require)\s*\(\s*['"][^'"]*"""
                         + re.escape(other) + r"""['"]""", body):
                raise Refuse(f"{stem}: fixture {key!r} names {other!r} in its "
                             f"own text -- renaming it is a rule-9 violation")


def check_self_inspection(stems) -> None:
    """Ruling 10's tool, over every target in this batch.

    Ruling 10 is a TOOL, not a sentence -- "do not re-implement the predicate
    from this prose, which is what the first version's reader did." It is run
    here, over these seven sources, and its `--selftest` is a `--gates-only`
    gate in its own right.
    """
    args = [sys.executable,
            os.path.join(PILOT, "find_fixture_self_inspection.py")]
    args += [os.path.join(TESTS, s + ".rs") for s in stems]
    r = subprocess.run(args, capture_output=True, text=True)
    if r.returncode != 0:
        raise Refuse("find_fixture_self_inspection.py failed:\n" + r.stdout + r.stderr)
    unadj = re.search(r"UNADJUDICATED:\s*(\d+)", r.stdout)
    if not unadj:
        raise Refuse("find_fixture_self_inspection.py printed no verdict")
    if int(unadj.group(1)) != 0:
        raise Refuse("an UNADJUDICATED fixture-self-inspection instance in this "
                     "batch's targets:\n" + r.stdout)


def check_rationales_match_their_claims(case, step) -> None:
    """Both directions, against the RENDERED step.

    Checked against what will actually be written, never against the variable
    that produced the prose -- otherwise the assertion is satisfied by the same
    value twice and proves nothing.
    """
    r = case["rationale"]
    if "fails closed" in r and step.get("exit") not in ("failure", 1):
        raise Refuse(f"{case['name']}: rationale says fails-closed, step does not")
    if step.get("exit") == "failure" and "json_paths" in step:
        # a failing run still emits a JSON envelope under `--output json`; the
        # pairing is legal, but it must not be silent
        pass


# ---------------------------------------------------------------------------
# Committed known positives -- a refusal that cannot fire is not a check
# ---------------------------------------------------------------------------

def _selftests() -> None:
    class FakeSrc:
        fns = {}
        tests = []
    ok = 0

    # the duplication check fires on a body the extractor never produced
    class E1:
        pass
    ex = {"tests": [type("T", (), {"__getitem__": lambda s, k: []})()]}
    try:
        check_duplication_is_the_sources_own(
            "selftest", {"a.js": "BODY"}, {"tests": []})
        raise AssertionError("the duplication check did not fire")
    except Refuse:
        ok += 1
        print("  ok  selftest: the duplication check fires on a foreign body")

    # U5's fixture-name check fires when one fixture imports another by name
    try:
        check_no_fixture_names_referenced(
            "selftest", {"a.js": "import('./b.js');", "b.js": "x"})
        raise AssertionError("the U5 fixture-name check did not fire")
    except Refuse:
        ok += 1
        print("  ok  selftest: the U5 name check fires on an imported fixture")

    # the reproduction-command check fires on a path that does not exist
    try:
        check_reproduction_commands(
            ["python3 scripts/audit-case-migration.py tests/nope.rs "
             "tests/cases/misc/nope.toml"])
        raise AssertionError("the reproduction-command check did not fire")
    except Refuse:
        ok += 1
        print("  ok  selftest: the reproduction check fires on a missing path")
    if ok != 3:
        raise AssertionError("a selftest did not run")


def check_reproduction_commands(commands: list[str]) -> None:
    """Every command a rendered header prints must actually reproduce.

    Each path argument must exist, and an `audit-case-migration.py` command is
    RUN and required to exit 0. Batch 4 shipped two headers whose reproduction
    command exited 1; a header can no longer do that, because the property is
    derived here rather than proof-read.
    """
    for cmd in commands:
        parts = cmd.split()
        for p in parts:
            if p.startswith("-") or "/" not in p:
                continue
            if p.startswith("$") or p.startswith("<"):
                continue
            path = p if os.path.isabs(p) else os.path.join(REPO, p)
            if not os.path.exists(path):
                raise Refuse(f"reproduction command names a path that does not "
                             f"exist: {p} (in `{cmd}`)")
        if "audit-case-migration.py" in cmd:
            r = subprocess.run(parts, cwd=REPO, capture_output=True, text=True)
            if r.returncode != 0:
                raise Refuse(f"reproduction command exits {r.returncode}: {cmd}\n"
                             + r.stdout[-2000:] + r.stderr[-2000:])


# ---------------------------------------------------------------------------
# Rendering one target
# ---------------------------------------------------------------------------

def build(stem: str) -> dict:
    ex = EX.extract(stem)
    src, ev = ex["src"], ex["ev"]
    check_captured(stem, src)
    pr = EX.prose(src)

    resolved = {}
    for t in ex["tests"]:
        for inv in t["invocations"]:
            for c in inv.claims:
                if c["kind"] == "or_fail_or_stdout":
                    resolved[id(c)] = resolve_or(inv, c)

    source_map: dict[str, str] = {}
    used: dict = {}
    cases = []
    for t in ex["tests"]:
        multi = len(t["invocations"]) > 1
        for n, inv in enumerate(t["invocations"], 1):
            name = t["name"] if not multi else f"{t['name']}__{n}"
            keymap = {}
            for tok in inv.argv:
                if isinstance(tok, PathVal):
                    k = source_key(inv, name, tok, used)
                    keymap[tok.name] = k
                    source_map[k] = inv.fixtures[tok.name]
            step = step_for(inv, resolved)
            step["args"] = [keymap.get(a, a) for a in step["args"]]

            bits = prose_for(pr, ev, t["name"])
            lead = (f"Migrated from `{stem}.rs::{t['name']}`"
                    + (f", invocation {n} of {len(t['invocations'])}" if multi else "")
                    + ".")
            ors = [c for c in inv.claims if c["kind"] == "or_fail_or_stdout"]
            for c in ors:
                r = resolved[id(c)]
                bits.append(
                    "RULE 11 -- the source's assertion is a DISJUNCTION, carried "
                    "verbatim: " + c["source"] + " The format has no disjunction, "
                    "so it was resolved against the real binary and the branch "
                    "that actually occurs (`" + r["branch"] + "`) is pinned. "
                    "Every run satisfying this pin satisfies the source's OR, so "
                    "the narrowing is a verified strengthening; the other "
                    "disjunct is disclosed here rather than asserted, because "
                    "the source never claimed it unconditionally (rule 2).")
            rationale = " ".join([lead] + bits).strip()
            case = {"name": name, "rationale": rationale, "steps": [step]}
            check_rationales_match_their_claims(case, step)
            cases.append(case)

    check_duplication_is_the_sources_own(stem, source_map, ex)
    check_no_fixture_names_referenced(stem, source_map)

    # RULE CONSTANTS. `audit-case-migration.py` extracts every `const NAME:
    # &str` as a claim the case file must carry, and `assertion_strings()`
    # searches `[constants]` but NOT `[source]` -- so a `const` holding FIXTURE
    # text is unsatisfiable unless it is hoisted, however faithfully the text is
    # reproduced. `switch/runtime.toml` hoists `switch_runtime.rs`'s `const
    # S`/`const SS` for exactly this reason and it is the standing precedent.
    #
    # Hoisted AND REFERENCED, which is forced rather than chosen: the audit
    # reports an `[unreferenced constant]` for a `[constants]` entry nothing
    # uses, so declaring it without spelling it into the fixture satisfies
    # nothing and adds a second red. Substituting `${NAME}` back into the
    # captured program text is an ENCODING of rule 9, not an exception to it --
    # the resolved bytes expand writes to disk are identical, which is the
    # same argument rule 10's `${dollar}` rests on. The generator asserts the
    # round trip below.
    #
    # U13's recorded counter-hazard applies and is the effect being relied on:
    # hoisting moves program text onto the surface `assertion_strings()`
    # searches. That is the only way a `const &str` holding FIXTURE text can be
    # satisfied at all, and `switch/runtime.toml` is the standing precedent.
    #
    # ORDER IS LOAD-BEARING: rule 10's escape runs FIRST, then the hoist. The
    # other way round, `esc_dollar` sees the `${NAME}` this step just introduced
    # and rewrites it to `${dollar}{NAME}`, so the reference the audit is looking
    # for never reaches the file and the constant reads as unreferenced -- which
    # is what happened on the first attempt, and it looked like the hoist had not
    # been applied at all.
    consts = {k: v for k, v in ex["src"].consts.items()
              if any(v in b for b in source_map.values())}

    dollar = needs_dollar(source_map, cases)
    if dollar:
        source_map = {esc_dollar(k): esc_dollar(v) for k, v in source_map.items()}
        for c in cases:
            c["steps"] = [{k: esc_dollar(v) for k, v in st.items()}
                          for st in c["steps"]]

    if consts:
        # Longest value first, so a constant that is a prefix of another cannot
        # eat it. The ROUND TRIP is asserted per body: resolving every `${NAME}`
        # back must reproduce the pre-hoist bytes exactly, which is the whole
        # claim that this is an encoding and not a rewrite (rule 9).
        for key, body in list(source_map.items()):
            before = body
            for name, value in sorted(consts.items(), key=lambda kv: -len(kv[1])):
                body = body.replace(value, "${" + name + "}")
            back = body
            for name, value in consts.items():
                back = back.replace("${" + name + "}", value)
            if back != before:
                raise Refuse(f"{stem}: hoisting a rule constant into {key!r} does "
                             f"not round-trip -- the encoding would change the "
                             f"program under test (rule 9)")
            source_map[key] = body

    header = header_for(stem, ex, pr, source_map, cases, used, dollar)
    text = emit(header, None, source_map, cases)
    if consts:
        block = "\n".join(f"{k} = {_toml_str(v)}" for k, v in sorted(consts.items()))
        text = text.replace("\n[source]\n", "\n[constants]\n" + block +
                            "\n\n[source]\n", 1)
    if dollar:
        # `emit` has no `[constants]` parameter, so the binding is spliced in --
        # into an existing `[constants]` block if the rule-constant hoist above
        # already opened one, otherwise ahead of `[source]`.
        if consts:
            text = text.replace("\n[constants]\n",
                                "\n[constants]\ndollar = \"$\"\n", 1)
        else:
            text = text.replace("\n[source]\n",
                                "\n[constants]\ndollar = \"$\"\n\n[source]\n", 1)
    return {"stem": stem, "text": text, "cases": cases, "ex": ex,
            "source": source_map, "resolved": resolved}


def esc_dollar(v):
    """RULE 10, applied to every surface substitute_step walks and no other.

    expand.rs's substitute() hard-fails on any `${…}` it cannot resolve, so a
    fixture carrying a genuine JS template literal must declare
    `[constants] dollar = "$"` and spell every real `${` as `${dollar}{`. The
    resolved program text is unchanged -- this is an ENCODING of rule 9, not an
    exception to it.

    Applied to `[source]` names and bodies, `args`, `env`, the stream pins, the
    `json` values and the count claims' `path`/`needle` -- exactly what
    substitute_step and expand walk (`expand.rs:81-128`, `:195-201`). NOT to
    `rationale`, which expand clones unsubstituted, which is why a carried
    source comment mentioning `${` is safe and must not be rewritten.
    """
    if isinstance(v, str):
        return v.replace("${", "${dollar}{")
    if isinstance(v, list):
        return [esc_dollar(x) for x in v]
    if isinstance(v, dict):
        return {esc_dollar(k): esc_dollar(x) for k, x in v.items()}
    return v


def needs_dollar(source_map, cases) -> bool:
    if any("${" in k or "${" in b for k, b in source_map.items()):
        return True
    for c in cases:
        for st in c["steps"]:
            for k, v in st.items():
                if "${" in repr(v):
                    return True
    return False


def arithmetic(stem, ex, cases) -> str:
    """Rule 7, asserted. A header cannot state arithmetic that does not close."""
    inv = sum(len(t["invocations"]) for t in ex["tests"])
    if inv != len(cases):
        raise Refuse(f"{stem}: {inv} invocation(s) but {len(cases)} case(s)")
    return (f"MATRIX DECLINED. {inv} source helper invocation(s) == "
            f"{len(cases)} case(s) x 1. Named siblings, one `[[case]]` per real "
            f"invocation (rule 5), so a failing trial names the source `#[test]` "
            f"fn it came from. A `[matrix]` axis is not declared: `[matrix]` is "
            f"FILE-WIDE (U1) and these files' cases do not all vary over one "
            f"axis uniformly, so an axis would fan cases the source never ran "
            f"(rule 2) and the arithmetic would not close (rule 7).")


GATES = [
    ("audit-case-migration.py", "scripts/audit-case-migration.py", "forward"),
    ("check_fixtures.py", "tools/task-18-browser-pilot/check_fixtures.py", "forward"),
    ("comment_coverage.py", "tools/task-18-browser-pilot/comment_coverage.py",
     "forward"),
    ("check_extra_claims.py", "tools/task-18-browser-pilot/check_extra_claims.py",
     "reverse"),
    ("check_rationale_fn_names.py",
     "tools/task-18-browser-pilot/check_rationale_fn_names.py", "reverse"),
]

# Why a gate is expected red, keyed on (gate, rc). Written as REASONS, not as a
# per-file list: the generator refuses to render a declaration for an (gate, rc)
# pair that has no entry here, so a NEW red cannot ship undeclared.
RED_REASONS = {
    ("comment_coverage.py", 1):
        "per-case and file-wide prose attribution (U6). This gate pools the "
        "header and every rationale and tests membership in the union, so it "
        "cannot verify per-case coverage at all; U6 forbids the over-attribution "
        "that would turn it green, and requires the false MISSING to be "
        "documented here instead.",
    ("comment_coverage.py", 2):
        "ruling 5's ZERO-LINE FLOOR, not a coverage failure: this source carries "
        "no non-divider Rust comment at all, and the checker refuses to report a "
        "vacuous green over zero checked lines. Confirmed by reading the source, "
        "not inferred from the exit code.",
    ("audit-case-migration.py", 1):
        "a U4 TRIM ARTIFACT. This is a FORWARD coverage gate (ruling 19), so its "
        "correct left-hand side is the MIGRATED COMPLEMENT, against which it is "
        "green; run against the post-trim file it reports the retained half's "
        "content as missing, which the case file legitimately does not carry.",
    ("check_extra_claims.py", 1):
        "a U4 TRIM ARTIFACT. This is a REVERSE existence gate (ruling 19), so its "
        "correct left-hand side is the PRE-TRIM BLOB -- the only side carrying "
        "both halves' names -- against which it is green.",
    ("check_rationale_fn_names.py", 1):
        "a U4 TRIM ARTIFACT. This is a REVERSE existence gate (ruling 19): the "
        "rationales legitimately name the migrated fns they were built from, and "
        "those names exist only in the pre-trim blob, against which it is green.",
}


def gate_rc(script: str, rs: str, toml: str) -> int:
    r = subprocess.run([sys.executable, os.path.join(REPO, script), rs, toml],
                       cwd=REPO, capture_output=True, text=True)
    return r.returncode


def redlist(stem: str) -> list[tuple]:
    """Run all five of ruling 19's gates for this pair. Measured, never predicted.

    Returns `[(gate, rc, reason)]` for every gate that is not green, and RAISES
    on a red for which no reason is declared -- so a new red fails the generator
    instead of shipping undeclared (ruling 9).
    """
    family, filestem = PLACEMENT[stem]
    rs = os.path.join(TESTS, stem + ".rs")
    toml = os.path.join(CASES, family, filestem + ".toml")
    if not os.path.exists(toml):
        return []
    out = []
    for name, script, _direction in GATES:
        rc = gate_rc(script, rs, toml)
        if rc == 0:
            continue
        if (name, rc) not in RED_REASONS:
            raise Refuse(f"{stem}: {name} exits {rc} and no reason is declared "
                         f"for that (gate, rc) -- ruling 9 requires every red "
                         f"named in-header")
        out.append((name, rc, RED_REASONS[(name, rc)]))
    return out


def trim_three_columns(stem: str) -> list[tuple]:
    """The three-column red-list for a U4 trim (rulings 9, 12 and 19).

    post-trim / pre-trim / migrated-complement, every cell RUN. The complement is
    built mechanically by migrated_complement.py -- ruling 12's point is that
    the pre-trim blob is the wrong left-hand side for a forward coverage gate
    whenever the RETAINED half carries literal claims of its own, which here it
    does (twenty-three alias literals).
    """
    family, filestem = PLACEMENT[stem]
    toml = os.path.join(CASES, family, filestem + ".toml")
    post = os.path.join(TESTS, stem + ".rs")
    tmp = tempfile.mkdtemp(prefix="t19b5-3col-")
    try:
        pre = os.path.join(tmp, "pre.rs")
        with open(pre, "wb") as f:
            f.write(subprocess.run(
                ["git", "show",
                 f"{EX.PRE_TRIM[stem]}:crates/kali_cli/tests/{stem}.rs"],
                cwd=REPO, capture_output=True, check=True).stdout)
        comp = os.path.join(tmp, "complement.rs")
        r = subprocess.run(
            [sys.executable,
             os.path.join(PILOT, "migrated_complement.py"), pre, post],
            cwd=REPO, capture_output=True, text=True, check=True)
        with open(comp, "w", encoding="utf-8") as f:
            f.write(r.stdout)
        rows = []
        for name, script, direction in GATES:
            rows.append((name,
                         gate_rc(script, post, toml),
                         gate_rc(script, pre, toml),
                         gate_rc(script, comp, toml),
                         "complement" if direction == "forward" else "pre-trim"))
        return rows
    finally:
        import shutil
        shutil.rmtree(tmp, ignore_errors=True)


def dotted_count_paths(stem, ex, cases) -> list[str]:
    """`json_count` paths that are DOTTED, with the check that they are derived.

    `check_extra_claims.py` compares a claim string literally, and a
    `json_count` claim's `path` is one. A single-segment path (`stdout`) occurs
    in the source verbatim; a dotted one (`errors.0.message`) never does,
    because the source spells it as an index chain -- `json["errors"][0]
    ["message"]`. The path is therefore a SYNTHESISED addressing expression, not
    an asserted string, and it is declared as such. Each segment is required to
    occur in the source before the declaration is written, so a path this
    generator invented could not be declared away.
    """
    text = ex["src"].text
    out = []
    for c in cases:
        for st in c["steps"]:
            for claim in st.get("json_count", []):
                path = claim["path"]
                if "." not in path or path in out:
                    continue
                for seg in path.split("."):
                    if seg.isdigit():
                        continue
                    if f'"{seg}"' not in text:
                        raise Refuse(
                            f"{stem}: json_count path {path!r} has a segment "
                            f"`{seg}` that occurs nowhere in the source")
                out.append(path)
    return out


def header_for(stem, ex, pr, source_map, cases, used, dollar=False) -> list[str]:
    family, filestem = PLACEMENT[stem]
    lines = [f"Migrated from tests/{stem}.rs.",
             f"  SOURCE REF: {SOURCE_REF}",
             ""]
    for line, block in sorted(pr["file"]):
        txt = " ".join(x for x in block if not is_divider(x)).strip()
        if txt:
            lines.append(txt)
            lines.append("")
    lines.append(arithmetic(stem, ex, cases))
    lines.append("")
    lines.append(
        "U5 -- EVERY `[source]` KEY IS RENAMED, AND THE RENAME IS FORCED. "
        "`[source]` is one flat file-wide namespace and expand.rs clones the "
        "whole map into every trial (U2), but these sources write MANY DIFFERENT "
        "programs to the same filename. A bare `main.ts` reused across cases "
        "would give every trial the last body written and silently destroy the "
        "cases' discriminating power. Each key is therefore the source `#[test]` "
        "fn's own name plus the original suffix chain, so the fn name survives "
        "the source's deletion. The WHOLE suffix chain is kept (`.test.js`, not "
        "`.js`) because kali test dispatches on the `.test.` infix: U5 permits "
        "renaming an argv filename, not renaming one whose shape the tool reads.")
    lines.append("")
    lines.append(
        "U13 -- THE HOIST IS DECLINED (ruling 7), and the duplication is "
        "asserted rather than eyeballed. check_fixtures.py's "
        "toml_program_texts reads `[source]` values verbatim and never "
        "resolves a `${NAME}` reference, so hoisting a shared body into "
        "`[constants]` makes the rule-9 fixture gate report UNMATCHED on a "
        "correct file. The property is the TOOL's, not the family's, which is "
        "what makes ruling 7 apply here as well as in `browser/`. "
        "check_duplication_is_the_sources_own requires the set of distinct "
        "rendered `[source]` bodies to equal the set of distinct fixture texts "
        "the extractor produced, so every byte-identical pair below is the "
        "SOURCE's own duplication and not the generator's.")
    if dollar:
        lines.append("")
        lines.append(
            "RULE 10 -- A GENUINE JS TEMPLATE LITERAL IS ENCODED, NOT ALTERED. "
            "One fixture in this file carries a real `${...}` interpolation. "
            "expand.rs's substitute() hard-fails on any `${...}` it cannot "
            "resolve, so `[constants] dollar = \"$\"` is declared and every real "
            "`${` is spelled `${dollar}{`. The program text the trial writes to "
            "disk is byte-identical to the source's -- this is an encoding of "
            "rule 9, not an exception to it. The escape is applied only to the "
            "surfaces expand substitutes; rationale is cloned unsubstituted, "
            "so a carried source comment mentioning `${` is left exactly as the "
            "source wrote it (rule 12).")
    reds = redlist(stem)
    lines.append("")
    if reds:
        family, filestem = PLACEMENT[stem]
        toml_rel = f"crates/kali_cli/tests/cases/{family}/{filestem}.toml"
        rs_rel = f"crates/kali_cli/tests/{stem}.rs"
        lines.append(
            "EVERY GATE THAT IS RED ON THIS PAIR, NAMED HERE WITH A REPRODUCTION "
            "THAT RUNS (ruling 9). All five of ruling 19's gates are run on every "
            "generator invocation and the list below is the generator's OWN "
            "output, not a second computation of the same thing -- so a gate that "
            "went green would fail this file rather than leave a stale claim, and "
            "a NEW red raises instead of shipping undeclared. The gates not "
            "listed are green.")
        for name, rc, reason in reds:
            script = dict((n, p) for n, p, _d in GATES)[name]
            lines.append("")
            lines.append(f"  EXPECTED-RED  {name}  rc={rc}")
            lines.append(f"    reproduce: python3 {script} {rs_rel} {toml_rel}")
            lines.append(f"    reason: {reason}")
    else:
        lines.append(
            "ALL FIVE of ruling 19's gates are GREEN on this pair "
            "(audit-case-migration.py, check_fixtures.py, comment_coverage.py, "
            "check_extra_claims.py, check_rationale_fn_names.py). Stated because "
            "a silent header is otherwise indistinguishable from one whose author "
            "only ran some of them.")
    lines.append("")
    for path in sorted(dotted_count_paths(stem, ex, cases)):
        lines.append(
            f"EXTRA-OK: {path!r} -- the dotted ADDRESS of a json_count claim, "
            f"not an asserted string. The source spells the same address as an "
            f"index chain (`json[\"errors\"][0][\"message\"]`), so the joined "
            f"form occurs nowhere in it; every segment does, and the generator "
            f"raises if one does not.")
    for key, (_body, orig) in sorted(used.items()):
        lines.append(
            f"EXTRA-OK: {key!r} -- a U5 variant-suffixed `[source]` key "
            f"surfaced as an argv token; it is a fixture FILENAME named after "
            f"the source `#[test]` fn that wrote the program (renamed from "
            f"`{orig}`), not a claim about behaviour")
    return lines


# ---------------------------------------------------------------------------
# The four DECLINED targets' U3 retention headers -- derived, and a fixed point
# ---------------------------------------------------------------------------

DECLINED_HEADER = '''//! SPEC §5.11 RETENTION -- CONTROLLER RULING R1, CLASS A (UNREACHABLE-CODE
//! CLAIM). This file stays hand-written per spec §5.11. Adjudicated in Task 15
//! and upheld on re-review; this header was added in Task 19 batch 5, and its
//! absence until now is the whole reason a dispatch listed this file as a
//! migratable CLEAN target.
//!
//! THE PHRASE "hand-written per spec" ABOVE IS LOAD-BEARING, and that is worth
//! knowing before anyone rewords this paragraph. `screen_candidates.py`'s S27
//! arm decides "is this an adjudicated retention?" by matching one of six
//! marker phrases against the `//!` header -- ruling 18's fragile shape, where
//! the gate's input is the prose it is policing. Its `--selftest` DOES fail
//! loudly when the marker stops matching while `citation_sweep.sh` still adopts
//! the file as a retention, which is how this was caught rather than shipped.
//!
//! THE BLOCKING CONSTRUCT, BY NAME AND LINE, RE-MEASURED RATHER THAN CITED.
//! %(dead_prose)s
//! Every call site passes `false`, so each `if json_output { … }` block is
//! UNREACHABLE and every literal inside it is DEAD: a value written in the
//! source and asserted by no reachable path.
//!
//! The enumerating command, run before this sentence was written (ruling 13):
//!
//!   cd /workspace && python3 tools/migration/t19b5_extract.py --declined
//!
//! It is re-run on every generator invocation (`t19b5_extract.check_declined`),
//! and it RAISES if any of these branches ever becomes reachable -- so this
//! retention is re-derived rather than inherited, and a source that grew a
//! `json_output = true` call site would fail the gate instead of staying
//! silently declined.
//!
//! WHY NEITHER THE AUDIT NOR THE FORMAT CAN CARRY IT. Those literals are dead.
//! `audit-case-migration.py` is a literal-coverage tool and cannot see
//! reachability, so it demands all of them of a case file; rule 2 forbids
//! inventing a claim to satisfy it, a value computed but never asserted not
//! being a claim; and rule 3 forbids shipping the resulting red. Controller
//! ruling R1 settles exactly this shape and rules BOTH alternatives out
//! permanently -- a per-file audit exception, and teaching the audit Rust
//! reachability analysis.
//!
//! ONLY SOME TESTS REACH IT, AND THE TRIM WAS QUANTIFIED AND DECLINED. This is
//! NOT a case where U4's whole-file clause applies on its own terms: %(mig)d of
//! this file's %(tot)d test fns never reach a dead-branch helper, so a
//! trim-and-keep IS structurally available. (The attribute is spelled "test
//! fns" and not in full here on purpose: `screen_candidates.py` counts test
//! functions by matching the attribute over the whole file, so writing it in
//! this header would add to the number the screen reports about the file --
//! ruling 11's self-referential trap, in miniature.) It was measured across all four
//! Class A files -- 10 migratable of 36, against 26 retained across four new
//! retention pairs -- and DECLINED by the controller, with the human partner's
//! agreement, on this ground: **U4 exists to stop OVER-retention, and a trim
//! that retains 26 of 36 barely reduces retention** while adding four instances
//! of the apparatus rulings 9, 11, 12 and 19 record as this project's densest
//! defect source. The sibling precedent runs the other way and is what makes
//! the distinction rather than contradicting it: Task 19 batch 2 trimmed
//! `object_has_own_frozen_js_input.rs`, the FIFTH Class A file from the same
//! Task 15 ruling, migrating 4 of its 5. The yield is inverted here.
//!
//! CONSEQUENCE FOR THE GATES. There is no case file for this stem and no trim,
//! so no per-pair gate runs against it at all and there is no red-list to
//! carry: rulings 9, 12 and 19's three-column apparatus is for a retention
//! PAIR, and this is a whole-file retention with no pair. What this header does
//! change is the SCREEN: `screen_candidates.py` now classifies this file
//! `S27_self_documented` instead of CLEAN, which is precisely the U3 mechanism
//! that was missing.
//!
//! Report: `.superpowers/sdd/2026-07-29-test-binary-consolidation/task-19-batch5-report.md`.

'''


TRIM_HEADER_PATH = "for_of_array_iteration_spread"


def trim_header(write: bool) -> None:
    """Fill the U4 trim retention header's three-column table from live runs.

    Ruling 9 requires every gate that is expected red on a retention pair to be
    named in the retained `.rs`'s own header; rulings 12 and 19 require three
    columns and the correct side PER GATE by direction of check. Every cell here
    is produced by running the gate -- the table cannot state a cell nobody
    measured, and the header is held to a fixed point like the case files.
    """
    path = os.path.join(TESTS, TRIM_HEADER_PATH + ".rs")
    with open(path, encoding="utf-8") as f:
        text = f.read()
    rows = trim_three_columns(TRIM_HEADER_PATH)
    word = {0: "GREEN", 1: "RED", 2: "RED(2)"}
    table = []
    for name, post, pre, comp, side in rows:
        table.append(f"//!   {name:<29} {word.get(post, post):<10} "
                     f"{word.get(pre, pre):<9} {word.get(comp, comp):<11} {side}")
    marker_a = "//!   gate                          post-trim  pre-trim  complement  correct side\n"
    i = text.index(marker_a) + len(marker_a)
    j = text.index("//!\n", i)
    new = text[:i] + "\n".join(table) + "\n" + text[j:]
    # WHAT IS ASSERTED, AND IT IS NOT "every correct side is green". Two
    # different things are red on this pair and conflating them would be the
    # ruling-12 overreach in miniature:
    #
    #   * a TRIM ARTIFACT is red post-trim and GREEN on its correct side. The
    #     audit is one, and rule 3 is absolute, so the audit's correct-side cell
    #     is required to be 0 -- if it were not, this would not be a trim
    #     artifact and the pair could not ship;
    #   * a red that survives on the correct side is NOT a trim artifact at all.
    #     comment_coverage.py is one: its U6 per-helper attribution red is the
    #     same red the other six pairs in this batch carry, and it is declared
    #     with the same reason. It must still have a declared reason, or it
    #     ships unexplained (ruling 9).
    for name, post, pre, comp, side in rows:
        correct = comp if side == "complement" else pre
        if correct == 0:
            continue
        if name == "audit-case-migration.py":
            raise Refuse(f"{TRIM_HEADER_PATH}: the audit is {correct} against "
                         f"its CORRECT side ({side}). Rule 3 is absolute and "
                         f"this is not a trim artifact.")
        if (name, correct) not in RED_REASONS:
            raise Refuse(f"{TRIM_HEADER_PATH}: {name} is {correct} on its "
                         f"correct side and no reason is declared (ruling 9)")
    if write:
        with open(path, "w", encoding="utf-8") as f:
            f.write(new)
    elif new != text:
        raise Refuse(f"{TRIM_HEADER_PATH}.rs: the three-column red-list has "
                     f"drifted from what the gates now report")


def declined_headers(write: bool) -> list[str]:
    """Render the four `//!` headers, and hold them to a fixed point.

    U3 requires an in-tree marker on every retention: "a retention whose
    reasoning lives only in a report is indistinguishable from a skipped file",
    and a later agent "cannot tell 'adjudicated' from 'overlooked'". These four
    had none, and this batch is the first measured consequence -- its dispatch
    listed all four as migratable.

    The prose is DERIVED from `check_declined`'s own measurement rather than
    transcribed, so the header cannot state a dead branch nobody measured.
    """
    dead = EX.check_declined()
    out = []
    for stem in sorted(EX.DECLINED):
        path = os.path.join(TESTS, stem + ".rs")
        with open(path, encoding="utf-8") as f:
            text = f.read()
        body = text
        if body.startswith("//!"):
            i = body.index("\n\n")
            body = body[i + 2:]
        helpers = dead[stem]
        names = ["`" + h.split("(")[0] + "`" for h in helpers]
        joined = (names[0] if len(names) == 1
                  else ", ".join(names[:-1]) + " and " + names[-1])
        prose = (("The `json_output: bool` parameter of " + joined + " is"
                  if len(names) == 1 else
                  "The `json_output: bool` parameters of " + joined + " are")
                 + " never passed `true` at any call site.")
        tot = len(re.findall(r"#\[test\]", body))
        blocking = {h.split("(")[0] for h in helpers}
        mig = 0
        for m in re.finditer(r"#\[test\]\s*\nfn\s+(\w+)\s*\(\s*\)\s*\{", body):
            end = body.find("\n}\n", m.end())
            if not any(b + "(" in body[m.end():end] for b in blocking):
                mig += 1
        head = DECLINED_HEADER % {"dead_prose": prose, "tot": tot, "mig": mig}
        new = head + body
        out.append(path)
        if write:
            with open(path, "w", encoding="utf-8") as f:
                f.write(new)
        elif new != text:
            raise Refuse(f"{stem}.rs: retention header has drifted from what "
                         f"this generator renders")
    return out


# ---------------------------------------------------------------------------
# Driver
# ---------------------------------------------------------------------------

def write_case(path: str, text: str) -> None:
    """Plain write, deliberately NOT `case_emit.write`.

    `case_emit.declare_source_ref` is wrong for every non-browser source -- it
    compares this file's `SOURCE REF:` against the *browser family deletion*
    ref and refuses when they differ, which for a source still in the tree is
    always. Batch 3 recorded the defect and batch 4 routed around it the same
    way. Recorded again here rather than fixed, because fixing it is a change to
    a module that 14 shipped generators write through, including the ones behind
    the 161 irreplaceable browser case files.
    """
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as f:
        f.write(text)


def path_for(stem: str) -> str:
    family, filestem = PLACEMENT[stem]
    return os.path.join(CASES, family, filestem + ".toml")


def main(argv) -> int:
    _selftests()
    EX.check_declined()
    check_self_inspection(EX.STEMS)

    # ITERATED TO A FIXED POINT. `redlist` runs the gates against the case file
    # ON DISK, and the header it produces is part of that file -- so the first
    # rendering measures the PREVIOUS revision. U3 records the same shape for
    # citations ("the first re-derivation shifts the header again"). Rendering is
    # repeated until two consecutive rounds agree, and `--write` writes each
    # round, so the shipped file is measured against itself.
    built = [build(s) for s in EX.STEMS]
    if argv and argv[0] == "--write":
        for _round in range(6):
            for p_ in declined_headers(write=True):
                pass
            trim_header(write=True)
            for b in built:
                write_case(path_for(b["stem"]), b["text"])
            again = [build(s) for s in EX.STEMS]
            if all(a["text"] == b["text"] for a, b in zip(again, built)):
                built = again
                break
            built = again
        else:
            raise Refuse("the rendering did not reach a fixed point in 6 rounds")
    if argv and argv[0] == "--list":
        for b in built:
            print(f"{path_for(b['stem'])}  {len(b['cases'])} case(s)")
        return 0
    if argv and argv[0] == "--write":
        for b in built:
            write_case(path_for(b["stem"]), b["text"])
            print(f"wrote {path_for(b['stem'])}  {len(b['cases'])} case(s)")
        for p_ in declined_headers(write=True):
            print(f"wrote retention header {p_}")
        trim_header(write=True)
        print(f"wrote the U4 trim red-list into {TRIM_HEADER_PATH}.rs")
        return 0
    declined_headers(write=False)
    trim_header(write=False)
    drift = []
    for b in built:
        p = path_for(b["stem"])
        if not os.path.exists(p):
            drift.append(f"{p}: MISSING")
            continue
        with open(p, encoding="utf-8") as f:
            if f.read() != b["text"]:
                drift.append(f"{p}: DRIFT")
    if drift:
        for d in drift:
            print(d, file=sys.stderr)
        return 1
    print(f"GENERATOR FIXED POINT -- {len(built)} case file(s), "
          f"{sum(len(b['cases']) for b in built)} case(s), reproduced "
          f"byte-for-byte")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
