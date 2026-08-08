r"""Live capture against the real built `kali` binary (Task 18 batch 3, U9).

Every exact `stdout` / `json.*` value written into this batch's case files is
read back from a real process run here, never hand-computed. `json_stdout()` is
the one that matters most: source's claim is usually
`json["stdout"].as_str().contains(X)`, and a nested `json` leaf has no
substring form in the case-file format, so the migration must pin the leaf
exactly -- which is only legitimate when the exact value came from the binary.

Usage as a library:
    from capture import run, json_envelope, json_stdout
"""
import json as _json
import os
import subprocess
import sys
import tempfile

KALI = os.environ.get('KALI_BIN', '/workspace/.cache/cargo-target/debug/kali')


def run(files, args, env=None):
    """files: {relpath: body} written into a fresh temp dir; args: argv after
    the binary. Returns (returncode, stdout_text, stderr_text)."""
    with tempfile.TemporaryDirectory() as d:
        for rel, body in files.items():
            path = os.path.join(d, rel)
            parent = os.path.dirname(path)
            if parent:
                os.makedirs(parent, exist_ok=True)
            with open(path, 'w', encoding='utf-8') as f:
                f.write(body)
        full = dict(os.environ)
        if env:
            full.update(env)
        p = subprocess.run([KALI] + list(args), cwd=d, env=full,
                           capture_output=True, text=True)
        return p.returncode, p.stdout, p.stderr


def json_envelope(files, args, env=None):
    rc, out, err = run(files, args, env)
    try:
        return rc, _json.loads(out), err
    except _json.JSONDecodeError:
        raise SystemExit(f'not JSON (rc={rc}):\nSTDOUT {out!r}\nSTDERR {err!r}')


def json_stdout(files, args, env=None):
    rc, env_json, err = json_envelope(files, args, env)
    return env_json['stdout']


if __name__ == '__main__':
    body = sys.stdin.read()
    name = sys.argv[1]
    argv = sys.argv[2:]
    rc, out, err = run({name: body}, argv,
                       {'KALI_BROWSER_BUNDLE_HARNESS_COMMAND': 'node'})
    print(f'rc={rc}\nSTDOUT {out!r}\nSTDERR {err!r}')
