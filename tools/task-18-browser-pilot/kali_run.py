import os
import subprocess
import tempfile
import json

KALI = "/workspace/.cache/cargo-target/debug/kali"


def run_kali(files, args, env=None, cwd_extra_env=None):
    """files: dict[relpath] = content, written into a fresh temp dir.
    args: list of argv (after the binary), executed with cwd=tempdir.
    env: dict merged into a stripped-down environment.
    Returns (exit_code, stdout_bytes, stderr_bytes)."""
    with tempfile.TemporaryDirectory() as d:
        for rel, content in files.items():
            path = os.path.join(d, rel)
            os.makedirs(os.path.dirname(path), exist_ok=True) if os.path.dirname(path) else None
            with open(path, "w") as f:
                f.write(content)
        full_env = dict(os.environ)
        if env:
            full_env.update(env)
        proc = subprocess.run(
            [KALI] + args, cwd=d, env=full_env, capture_output=True
        )
        return proc.returncode, proc.stdout, proc.stderr, d


def run_kali_keep(files, args, env=None):
    """Like run_kali but returns the tempdir path (caller must clean up) so
    later steps (e.g. reading an emitted app.meta.json) can inspect it."""
    d = tempfile.mkdtemp()
    for rel, content in files.items():
        path = os.path.join(d, rel)
        parent = os.path.dirname(path)
        if parent:
            os.makedirs(parent, exist_ok=True)
        with open(path, "w") as f:
            f.write(content)
    full_env = dict(os.environ)
    if env:
        full_env.update(env)
    proc = subprocess.run([KALI] + args, cwd=d, env=full_env, capture_output=True)
    return proc.returncode, proc.stdout, proc.stderr, d
