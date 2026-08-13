#!/usr/bin/env python3
"""Run a fixture through the real `kali` and report exit/stdout/stderr.

U9: every migrated case's expected values are captured from the real binary,
per case, never hand-computed. Reproduces what the case runner does -- a fresh
temp dir per trial, `current_dir` set to it, `[source]` files written into it --
so a value captured here is a value the runner will see.

Usage:  capture.py <spec.json>
  spec: [ {"label":..., "source":{name:body,...}, "args":[...], "env":{...}} ]
"""
import json, os, subprocess, sys, tempfile

KALI = "/workspace/.cache/cargo-target/debug/kali"

def run(spec):
    with tempfile.TemporaryDirectory() as d:
        for name, body in spec.get("source", {}).items():
            p = os.path.join(d, name)
            os.makedirs(os.path.dirname(p), exist_ok=True)
            with open(p, "w") as f:
                f.write(body)
        env = dict(os.environ)
        env.update(spec.get("env", {}))
        r = subprocess.run([KALI] + spec["args"], cwd=d, capture_output=True, env=env)
    return {
        "label": spec["label"],
        "code": r.returncode,
        "success": r.returncode == 0,
        "stdout": r.stdout.decode("utf-8", "replace"),
        "stderr": r.stderr.decode("utf-8", "replace"),
        "stdout_is_utf8": r.stdout.decode("utf-8", "replace").encode("utf-8") == r.stdout,
    }

if __name__ == "__main__":
    specs = json.load(open(sys.argv[1]))
    out = [run(s) for s in specs]
    json.dump(out, sys.stdout, indent=1)
