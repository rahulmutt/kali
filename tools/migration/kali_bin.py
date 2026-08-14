#!/usr/bin/env python3
r"""Locating the built `kali` binary, for the generators that re-run it.

Two Task 19 generators are not pure renderers: `gen_task19_batch4.py` re-runs
the real binary for its rule-11 cross-stream resolutions and for the U2 policy
positive control, and `gen_task19_batch5.py` re-runs it for all eighteen of its
rule-11 disjunction resolutions. Both are CHECK-direction gates, so they must
fail loudly when the binary is missing rather than skip -- a control that
silently does not run is indistinguishable from a control that was deleted.

WHY THIS MODULE EXISTS. Both generators used to resolve the binary at
`$REPO/.cache/cargo-target/debug/kali`. That path is not a property of this
repository: it comes from a machine-local `~/.cargo/config.toml` in one dev
container, and there is no `.cargo/` directory in the tree. On any other machine
-- a fresh clone, another contributor's checkout, a CI runner -- cargo builds to
`./target` and the hardcoded path is simply wrong. `gen_task19_batch5.py` had no
existence check at all behind it, so it failed with a bare `FileNotFoundError`
from `subprocess.run` naming a directory the reader has no reason to recognise.

The resolution order below is `gen_task19_batch4.py`'s original chain with the
hardcoded path replaced by DERIVED target directories, `cargo metadata` first:

    1. $CARGO_BIN_EXE_kali   -- what Cargo itself sets for integration tests
    2. $KALI_BIN             -- the explicit override
    3. $CARGO_TARGET_DIR/debug/kali
    4. `cargo metadata --format-version 1 --no-deps` .target_directory,
       + /debug/kali -- authoritative, and honours .cargo/config.toml wherever
       it lives, including the dev container's `.cache/cargo-target`
    5. $REPO/target/debug/kali -- cargo's default, for when cargo is not on PATH

`require()` raises `KaliBinMissing` naming EVERY candidate it looked at and why
each one did not answer, so the failure says what to do about it.
"""

from __future__ import annotations

import json
import os
import subprocess

__all__ = ["KaliBinMissing", "find", "require", "search_report"]


class KaliBinMissing(RuntimeError):
    """No `kali` binary at any candidate location. The message lists them."""


# repo -> resolved path. Only successful resolutions are cached; a failure is
# re-derived so that building the binary mid-run is picked up.
_CACHE: dict[str, str] = {}


def _metadata_target_dir(repo: str) -> tuple[str | None, str | None]:
    """(target_directory, None) or (None, why it could not be read)."""
    try:
        r = subprocess.run(
            ["cargo", "metadata", "--format-version", "1", "--no-deps"],
            cwd=repo, capture_output=True, text=True)
    except OSError as e:
        return None, f"`cargo metadata` could not be run: {e}"
    if r.returncode != 0:
        first = next((ln for ln in r.stderr.splitlines() if ln.strip()), "")
        return None, f"`cargo metadata` exited {r.returncode}: {first.strip()}"
    try:
        return json.loads(r.stdout)["target_directory"], None
    except (ValueError, KeyError) as e:
        return None, f"`cargo metadata` output carried no .target_directory: {e}"


def _candidates(repo: str):
    """Yield (label, path-or-None, note-or-None) in resolution order.

    A generator on purpose: `find` stops at the first hit, so `cargo metadata`
    is not spawned when an env var already answers.
    """
    for var in ("CARGO_BIN_EXE_kali", "KALI_BIN"):
        v = os.environ.get(var)
        yield f"${var}", (v or None), (None if v else "unset")

    td = os.environ.get("CARGO_TARGET_DIR")
    yield ("$CARGO_TARGET_DIR/debug/kali",
           os.path.join(td, "debug", "kali") if td else None,
           None if td else "unset")

    md, why = _metadata_target_dir(repo)
    yield ("`cargo metadata --format-version 1 --no-deps` .target_directory "
           "+ /debug/kali",
           os.path.join(md, "debug", "kali") if md else None,
           why)

    yield ("cargo's default target dir",
           os.path.join(repo, "target", "debug", "kali"),
           None)


def find(repo: str) -> str | None:
    """The first candidate that exists, or None. Never raises."""
    hit = _CACHE.get(repo)
    if hit and os.path.exists(hit):
        return hit
    for _label, path, _note in _candidates(repo):
        if path and os.path.exists(path):
            _CACHE[repo] = path
            return path
    return None


def search_report(repo: str) -> str:
    """Every candidate and why it did not answer. Used in failure messages."""
    lines = []
    for i, (label, path, note) in enumerate(_candidates(repo), 1):
        if path is None:
            lines.append(f"  {i}. {label} -- {note or 'not available'}")
        elif os.path.exists(path):
            lines.append(f"  {i}. {label} -> {path} -- EXISTS")
        else:
            lines.append(f"  {i}. {label} -> {path} -- does not exist")
    return "\n".join(lines)


def require(repo: str, why: str) -> str:
    """The binary's path, or raise `KaliBinMissing` naming the whole search.

    `why` states what cannot run without it, so the message is actionable
    rather than merely true.
    """
    hit = find(repo)
    if hit:
        return hit
    raise KaliBinMissing(
        f"{why}: no `kali` binary found. Looked for, in resolution order:\n"
        f"{search_report(repo)}\n"
        f"  Build it with `cargo build -p kali_cli --bin kali`, or set "
        f"KALI_BIN=/path/to/kali.")


if __name__ == "__main__":  # a one-line answer to "where does this resolve?"
    import sys
    _repo = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)),
                                         "..", ".."))
    _hit = find(_repo)
    if _hit:
        print(_hit)
    else:
        print(f"no `kali` binary found. Looked for:\n{search_report(_repo)}",
              file=sys.stderr)
        sys.exit(1)
