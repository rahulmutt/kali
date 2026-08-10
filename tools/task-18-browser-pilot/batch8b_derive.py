#!/usr/bin/env python3
r"""`gen_batch8b.py --derive` -- the U2 measurements batch 8B's splits rest on.

Every batch-8B case file header cites this command by name, so it has to run and
it has to GATE (U12): it exits non-zero when any conclusion it records changes.
The declared answers live in `DECLARED` below and are compared against what this
run actually measures, which is ruling 15's first and only durable answer to a
figure -- declare it and gate it, from inside the loop that produces it.

TWO MEASUREMENTS, AND THE SECOND IS THE FIRST'S CONTROL.

D1 -- CAN A `#[path]` CARRIER'S `run` AND `test` HALVES SHARE A CASE FILE?
For each `#[test]` fn of the four `runtime_summary_fallback_*` carriers, run the
real binary on that fn's own argv and environment three times: in a directory
holding only its own fixture (what the source's `tempdir` held), one holding its
own half's fixtures, and one holding BOTH halves' (what a shared file-wide
`[source]` produces). `payload.runtimeMs` is masked because nothing pins it and
it is a clock; everything else is compared byte for byte. A difference anywhere
means the fold changes what the binary does and the halves must split.

D2 -- IS A CASE DISARMED BY A LEAKED `kali.json`?
With the other half's manifest present and the flag under test REMOVED, do the
case's pins still hold? If yes, a shared file would leave the flag unverified:
the manifest supplies the claim, no literal is dropped (so
`audit-case-migration.py` cannot see it) and the trial still passes (so
`cargo test` cannot either).

WHY D2 IS RUN EVEN FOR THE STEMS THAT DO NOT SPLIT. A zero nobody tried to make
non-zero is worth nothing. D1 returns zero differences, so the same instrument is
pointed at a case that IS disarmable and must report DISARMED. Row `control` in
the table below is exactly that: a `runtime_summary_fallback` case, which has no
manifest anywhere in its source, run with one added. It comes back DISARMED,
which is what makes D1's zero a statement about the fold rather than about the
harness.

AND THE SANDBOX ROW IS RUN PER CASE SHAPE, NOT PER TARGET. Its JSON cases pin
`errors[0].context.origin`, which no manifest can supply; its TEXT cases pin only
stderr substrings, every one of which a manifest does supply. Deriving from the
JSON cases alone answers "no split needed" and is wrong. `[source]` is file-wide
with no per-case opt-out, so one disarmed case disarms the file.
"""

import json
import os
import re
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
TESTS = os.path.join(REPO, "crates/kali_cli/tests")

from batch8b_capture import kali_bin  # noqa: E402
from batch8b_extract import (  # noqa: E402
    HARNESS_ENV, fn_body, literals, manifest_body, policy_body, summary_fallback_rows,
)

SF_STEMS = [f"runtime_summary_fallback_{e}_input" for e in ("js", "jsx", "ts", "tsx")]

# The measured answers, declared so a change is a failure rather than a surprise.
DECLARED = {
    "d1_probed": 117,
    "d1_differing": 0,
    "d2": {
        "runtime_wasm_threads_js_input": "DISARMED",
        "wasm_threads_max_threads_harness": "DISARMED",
        "wasm_threads_browser_surface": "DISARMED",
        "runtime_spawned_process_budget_js_input": "DISARMED",
        "runtime_sandbox_json_cases": "DISCRIMINATES",
        "runtime_sandbox_text_cases": "DISARMED",
        "control_summary_fallback": "DISARMED",
    },
    "or_group_cells": 16,
    "or_group_both_true": 16,
}


def run(files, argv, env=None):
    d = tempfile.mkdtemp(prefix="b8b-derive-")
    try:
        for rel, body in files.items():
            with open(os.path.join(d, rel), "w") as fh:
                fh.write(body)
        environ = dict(os.environ)
        environ.pop(HARNESS_ENV, None)
        environ.update(env or {})
        proc = subprocess.run([kali_bin()] + list(argv), cwd=d, env=environ,
                              capture_output=True)
        # RAW, deliberately. D1 masks for its comparison; D2 and D3 parse the
        # JSON, and `"runtimeMs":<masked>` is not valid JSON -- masking here made
        # every `jget` return "<not-json>" and every D2 pin fail. Caught by the
        # probe's own "do the case's own pins hold?" precondition, which exists
        # so a verdict is never reported by an instrument measuring nothing.
        return (proc.returncode,
                proc.stdout.decode("utf-8", "replace"),
                proc.stderr.decode("utf-8", "replace"))
    finally:
        shutil.rmtree(d, ignore_errors=True)


def masked(observation):
    rc, out, err = observation
    return rc, mask(out), mask(err)


def mask(text):
    """Mask only what nothing pins: the wall-clock field and temp paths."""
    text = re.sub(r'"runtimeMs":\d+', '"runtimeMs":<masked>', text)
    return re.sub(r"/tmp/[A-Za-z0-9_./-]+", "<tmp>", text)


def jget(stdout, path):
    try:
        value = json.loads(stdout)
    except Exception:
        return "<not-json>"
    for part in path.split("."):
        if isinstance(value, list):
            value = value[int(part)]
        elif isinstance(value, dict):
            if part not in value:
                return "<missing>"
            value = value[part]
        else:
            return "<missing>"
    return value


def d1():
    probed = differing = 0
    detail = []
    for stem in SF_STEMS:
        rows, carrier, subs, writer, fixture = summary_fallback_rows(stem)
        halves = {}
        for r in rows:
            halves.setdefault(r["half"], {})[r["file"]] = fixture
        merged = {}
        for files in halves.values():
            merged.update(files)
        for r in rows:
            probed += 1
            env = {HARNESS_ENV: r["harness"]}
            alone = masked(run({r["file"]: fixture}, r["argv"], env))
            own = masked(run(halves[r["half"]], r["argv"], env))
            both = masked(run(merged, r["argv"], env))
            if not (alone == own == both):
                differing += 1
                detail.append(f"{stem}/{r['half']}::{r['fn']}")
    return probed, differing, detail


def _holds(observation, pins):
    rc, out, err = observation
    for kind, path, expected in pins:
        if kind == "rc" and rc != expected:
            return False
        if kind == "json" and jget(out, path) != expected:
            return False
        if kind == "jsub":
            got = jget(out, path)
            if not isinstance(got, str) or expected not in got:
                return False
        if kind == "outsub" and expected not in out:
            return False
        if kind == "errsub" and expected not in err:
            return False
    return True


def d2_case(label, files, argv, argv_without_flag, pins, manifest, env=None):
    own = run(files, argv, env)
    leaked = dict(files)
    leaked["kali.json"] = manifest
    without = run(leaked, argv_without_flag, env)
    if not _holds(own, pins):
        raise AssertionError(
            f"{label}: the case's own pins do not hold on its own argv -- the probe is "
            "measuring the wrong thing, so its verdict means nothing")
    verdict = "DISARMED" if _holds(without, pins) else "DISCRIMINATES"
    print(f"  {label:44s} {verdict}")
    return verdict


def d2():
    out = {}
    sandbox = open(os.path.join(TESTS, "browser_runtime_sandbox_js_input.rs")).read()
    policy = policy_body(sandbox)
    mani_plain = manifest_body(sandbox)
    mani_threads = manifest_body(
        open(os.path.join(TESTS, "browser_wasm_threads_browser_surface.rs")).read())

    out["runtime_wasm_threads_js_input"] = d2_case(
        "runtime_wasm_threads_js_input",
        {"main.js": "console.log('browser wasm threads ok');\n"},
        ["--output", "json", "run", "--api", "browser", "--wasm-threads",
         "--max-threads", "1", "--max-spawned-processes", "0", "main.js"],
        ["--output", "json", "run", "--max-threads", "1",
         "--max-spawned-processes", "0", "main.js"],
        [("rc", None, 0), ("json", "payload.hostContract", "browser-requested"),
         ("json", "payload.runtimeBackend", "browser-harness")],
        mani_threads, {HARNESS_ENV: "node"})

    out["wasm_threads_max_threads_harness"] = d2_case(
        "wasm_threads_max_threads_harness",
        {"main.js": "console.log('browser max threads ok');\n"},
        ["--output", "json", "run", "--api", "browser", "--wasm-threads",
         "--max-threads", "1", "--max-spawned-processes", "0", "main.js"],
        ["--output", "json", "run", "--wasm-threads", "--max-threads", "1",
         "--max-spawned-processes", "0", "main.js"],
        [("rc", None, 0), ("json", "payload.hostContract", "browser-requested"),
         ("json", "payload.runtimeBackend", "browser-harness")],
        mani_plain, {HARNESS_ENV: "node"})

    out["wasm_threads_browser_surface"] = d2_case(
        "wasm_threads_browser_surface",
        {"app.js": "let value = 1 + 2; value;\n"},
        ["--output", "json", "check", "--api", "browser", "--wasm-threads", "app.js"],
        ["--output", "json", "check", "app.js"],
        [("rc", None, 5), ("json", "errors.0.code", "E5506"),
         ("jsub", "errors.0.message", "runtime profile")],
        mani_threads)

    out["runtime_spawned_process_budget_js_input"] = d2_case(
        "runtime_spawned_process_budget_js_input",
        {"main.js": "console.log('browser spawned process budget ok');\n"},
        ["--output", "json", "run", "--api", "browser",
         "--max-spawned-processes", "0", "main.js"],
        ["--output", "json", "run", "--max-spawned-processes", "0", "main.js"],
        [("rc", None, 0), ("json", "payload.hostContract", "browser-requested"),
         ("json", "payload.runtimeBackend", "browser-harness")],
        mani_plain, {HARNESS_ENV: "node"})

    sb = {"main.js": "console.log('browser run');", "kali.policy.json": policy}
    out["runtime_sandbox_json_cases"] = d2_case(
        "runtime_sandbox_* (JSON cases)", sb,
        ["--output", "json", "run", "--api", "browser", "--sandbox",
         "kali.policy.json", "main.js"],
        ["--output", "json", "run", "--sandbox", "kali.policy.json", "main.js"],
        [("rc", None, 1), ("json", "errors.0.code", "E5506"),
         ("json", "errors.0.context.origin", "cli"),
         ("jsub", "errors.0.message", "standalone browser runtime contract")],
        mani_plain, {HARNESS_ENV: "node"})

    out["runtime_sandbox_text_cases"] = d2_case(
        "runtime_sandbox_* (TEXT cases)", sb,
        ["run", "--api", "browser", "--sandbox", "kali.policy.json", "main.js"],
        ["run", "--sandbox", "kali.policy.json", "main.js"],
        [("rc", None, 1), ("errsub", None, "E5506"),
         ("errsub", None, "standalone browser runtime contract"),
         ("errsub", None, "selected host contract: browser-requested")],
        mani_plain, {HARNESS_ENV: "node"})

    rows, carrier, subs, writer, fixture = summary_fallback_rows(SF_STEMS[0])
    probe = next(r for r in rows if r["half"] == "run" and "--output" not in r["argv"])
    out["control_summary_fallback"] = d2_case(
        "control: a summary_fallback case + a manifest",
        {probe["file"]: fixture},
        probe["argv"], [a for a in probe["argv"] if a not in ("--api", "browser")],
        [("rc", None, 0), ("outsub", None, probe["claims"][1][1])],
        mani_plain, {HARNESS_ENV: probe["harness"]})
    return out


def d3():
    """Ruling 17: resolve the one OR-shaped assertion over every cell."""
    text = open(os.path.join(TESTS, "browser_wasm_threads_browser_surface.rs")).read()
    helper, _ = fn_body(text, "assert_browser_wasm_threads_rejection")
    needles = re.findall(r'\.contains\("((?:[^"\\]|\\.)*)"\)', helper)
    needles = [n for n in needles if n != "E5506"]
    mani = manifest_body(text)
    src = literals(fn_body(text, "assert_browser_wasm_threads_rejection_for_command")[0])
    src = [v for v in src if v.startswith("let value")]
    assert len(src) == 1, src
    cells = both = 0
    for command, extra in (("check", []), ("build", ["--bundle"])):
        for name in ("app.js", "app.ts", "app.jsx", "app.tsx"):
            for mode in ("explicit", "manifest"):
                files = {name: src[0]}
                argv = [command] + extra
                if mode == "explicit":
                    argv += ["--api", "browser", "--wasm-threads"]
                else:
                    files["kali.json"] = mani
                argv.append(name)
                _, _, err = run(files, argv)
                cells += 1
                if all(n in err for n in needles):
                    both += 1
    print(f"  disjuncts {needles!r}: both true on {both} of {cells} cells")
    return cells, both, needles


def main():
    print("D1 -- can a `#[path]` carrier's run/test halves share a case file?")
    probed, differing, detail = d1()
    print(f"  {probed} tests probed; {differing} whose output differs between "
          "isolated / own-half / both-halves directories")
    for line in detail[:10]:
        print(f"    DIFFERS: {line}")

    print("\nD2 -- is a case disarmed by a leaked `kali.json`?")
    verdicts = d2()

    print("\nD3 -- rule 11 / ruling 17: the one OR-shaped source assertion")
    cells, both, needles = d3()

    problems = []
    if probed != DECLARED["d1_probed"]:
        problems.append(f"D1 probed {probed}, declared {DECLARED['d1_probed']}")
    if differing != DECLARED["d1_differing"]:
        problems.append(f"D1 found {differing} differing, declared {DECLARED['d1_differing']}")
    for key, want in DECLARED["d2"].items():
        got = verdicts.get(key)
        if got != want:
            problems.append(f"D2 {key}: measured {got}, declared {want}")
    if (cells, both) != (DECLARED["or_group_cells"], DECLARED["or_group_both_true"]):
        problems.append(
            f"D3 measured {both}/{cells}, declared "
            f"{DECLARED['or_group_both_true']}/{DECLARED['or_group_cells']}")

    print()
    if problems:
        for p in problems:
            print(f"DERIVE FAILED: {p}")
        print("A conclusion this batch's splits rest on has changed. The case files' "
              "U2 sections are now describing a tree that no longer behaves that way.")
        return 1
    print("DERIVE OK -- every measurement matches the answer declared in this file, "
          "including the known positive that keeps D1's zero honest.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
