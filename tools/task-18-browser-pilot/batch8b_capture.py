#!/usr/bin/env python3
r"""Live capture for Task 18 batch 8B (U9, rule 8, rule 9).

Every value this batch PINS that the source only asserted a substring of comes
from running the real `kali` binary, never from hand-computation. Controller
ruling 3 forces exact pins for `json` string leaves (the format has no
json-substring key), and rule 8's discipline -- execute the real code, do not
simulate it -- applies to a captured envelope exactly as it does to a
`format!`.

WHAT IS RECORDED, AND WHY THE INPUTS ARE RECORDED WITH THE OUTPUT.
`batch8b_captures.py` stores, per capture id, the exact `[source]` map, argv and
environment the observation was taken under, alongside the observation. The
generator then re-derives those inputs from the `.rs` on every run and
`check_captured` raises when they differ. A capture taken before a source edit
therefore fails the generator rather than shipping a pin for a command the
source no longer issues -- which is the failure `check_captured` exists for in
batch 8A, hoisted here because 8B has 375 of them instead of 20.

WHAT IS DELIBERATELY NOT NORMALISED. `payload.runtimeMs` varies run to run and
nothing pins it, so it is left in the recorded stdout as observed; the generator
never reads it. Normalising it would make the record a processed artefact rather
than a transcript, and a reader could no longer tell what the binary actually
printed.

REALISM IS NOT FIDELITY (U9). These captures prove the case matches what the
binary does today. They prove nothing about whether a claim was dropped -- that
is `audit-case-migration.py`'s job, and the fidelity diff's.
"""

import json
import os
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))

from case_emit import require_debug_artifact  # noqa: E402

CAPTURE_FILE = os.path.join(HERE, "batch8b_captures.py")


def kali_bin():
    return require_debug_artifact(
        "kali", why="cargo build -p kali_cli --bin kali (the binary every capture runs)")


def observe(files, argv, env):
    """Run the real binary in a fresh dir holding exactly `files`."""
    d = tempfile.mkdtemp(prefix="b8b-capture-")
    try:
        for rel, body in files.items():
            path = os.path.join(d, rel)
            parent = os.path.dirname(path)
            if parent:
                os.makedirs(parent, exist_ok=True)
            with open(path, "w") as fh:
                fh.write(body)
        environ = dict(os.environ)
        # Removed rather than left alone: a developer shell that exports it
        # would silently change what every capture in this batch observes, and
        # the recorded environment would then be a lie about the run.
        environ.pop("KALI_BROWSER_BUNDLE_HARNESS_COMMAND", None)
        environ.update(env or {})
        proc = subprocess.run([kali_bin()] + list(argv), cwd=d, env=environ,
                              capture_output=True)
        out = proc.stdout.decode("utf-8", "replace")
        err = proc.stderr.decode("utf-8", "replace")
        extra = {}
        meta = os.path.join(d, "app", "app.meta.json")
        if os.path.exists(meta):
            extra["app/app.meta.json"] = json.load(open(meta))
        return {"rc": proc.returncode, "stdout": out, "stderr": err, "files": extra}
    finally:
        shutil.rmtree(d, ignore_errors=True)


def load():
    if not os.path.exists(CAPTURE_FILE):
        return {}
    import importlib
    sys.path.insert(0, HERE)
    mod = importlib.import_module("batch8b_captures")
    importlib.reload(mod)
    return mod.CAPTURES


def save(captures):
    lines = [
        '"""Live captures for Task 18 batch 8B -- GENERATED, do not hand-edit.',
        "",
        "Written by `python3 gen_batch8b.py --recapture`, which runs the real `kali`",
        "binary once per entry. `batch8b_capture.check_captured` re-derives every",
        "recorded input from the `.rs` on each generator run and raises on a mismatch,",
        "so an entry cannot outlive the command it describes.",
        '"""',
        "",
        "CAPTURES = {",
    ]
    for key in sorted(captures):
        lines.append(f"    {key!r}: {captures[key]!r},")
    lines.append("}")
    with open(CAPTURE_FILE, "w") as fh:
        fh.write("\n".join(lines) + "\n")
    print(f"wrote {CAPTURE_FILE} ({len(captures)} capture(s))")


class Captures:
    """The capture table, with the staleness check folded into every read."""

    def __init__(self, recapture=False):
        self.table = {} if recapture else load()
        self.recapture = recapture
        self.used = set()

    def get(self, key, files, argv, env):
        inputs = {"files": dict(files), "argv": list(argv), "env": dict(env or {})}
        rec = self.table.get(key)
        if self.recapture or rec is None:
            if not self.recapture:
                raise AssertionError(
                    f"no capture for {key!r}. Run `gen_batch8b.py --recapture` -- this "
                    "generator will not invent a pinned value, and will not fall back to "
                    "a stale one.")
            prior = self.table.get(key)
            if prior is not None and any(prior[f] != inputs[f]
                                         for f in ("files", "argv", "env")):
                raise AssertionError(
                    f"capture key {key!r} is claimed twice with DIFFERENT inputs. During a "
                    "recapture the second would silently overwrite the first and one case "
                    "would ship the other's observation. Make the key unique.\n"
                    f"  first:  {prior['argv']!r}\n  second: {inputs['argv']!r}")
            rec = dict(inputs)
            rec["observed"] = observe(files, argv, env)
            self.table[key] = rec
        else:
            for field in ("files", "argv", "env"):
                if rec[field] != inputs[field]:
                    raise AssertionError(
                        f"capture {key!r} is STALE: its recorded {field} is not what the "
                        f"source now derives.\n  recorded: {rec[field]!r}\n  derived:  "
                        f"{inputs[field]!r}\nRe-run `gen_batch8b.py --recapture`.")
        self.used.add(key)
        return rec["observed"]

    def json_leaf(self, key, files, argv, env, path):
        obs = self.get(key, files, argv, env)
        value = json.loads(obs["stdout"])
        for part in path.split("."):
            value = value[int(part)] if isinstance(value, list) else value[part]
        return value

    def finish(self):
        unused = sorted(set(self.table) - self.used)
        if unused and not self.recapture:
            raise AssertionError(
                "capture table holds entries no case reads: " + ", ".join(unused[:8]) +
                ("..." if len(unused) > 8 else "") +
                "\nA capture nothing reads is a value nothing re-verifies. Re-run "
                "`gen_batch8b.py --recapture`.")
        if self.recapture:
            save(self.table)
