#!/usr/bin/env python3
"""Resolve a `#[path = "..."] mod ...;` carrier into the files that hold its tests.

U10 names the failure this closes: for a target whose tests live behind a
`#[path]` submodule declaration, reading only the top-level `.rs` sees **zero**
`#[test]` fns and silently drops every one of them. `audit-case-migration.py`
grew its own `resolve_path_mods` for exactly that reason -- but the five
satellite gates in this directory (`check_extra_claims.py`,
`check_rationale_fn_names.py`, `check_fixtures.py`, `comment_coverage.py`,
`batch5_crosscheck.py`) each read `open(rs_path).read()` and so still had the
blind spot the audit script had already closed for itself.

That was harmless while no submodule-shaped target had been migrated. Enumerated
before this module was written:

    $ cd crates/kali_cli/tests && grep -ln '#\\[path' browser_*.rs | while read f; do
    >   s=${f#browser_}; s=${s%.rs}
    >   ls cases/browser/$s*.toml >/dev/null 2>&1 && echo "$f MIGRATED" || echo "$f none"
    > done

7 `browser_*.rs` carry a `#[path]` submodule declaration and, at the commit this
module was added, none of the 7 had a shipped case file. Task 18 batch 6B
migrates the first of them (`browser_non_literal_iterator_sources.rs`, 0
top-level `#[test]` fns and 90 across four submodules), which is what makes the
gap live rather than theoretical.

Rather than re-implement the resolution (three of this project's measurement
bugs came from a second implementation of a predicate that already existed),
this delegates to `audit-case-migration.py`'s own `resolve_path_mods`.
"""

import importlib.util
import os
from pathlib import Path

_AUDIT = os.path.join(
    os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))),
    "scripts",
    "audit-case-migration.py",
)


def audit_module():
    """`scripts/audit-case-migration.py` imported as a module."""
    spec = importlib.util.spec_from_file_location("audit", _AUDIT)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def submodule_paths(rs_path, mod=None, base=None):
    """Every `.rs` reachable from `rs_path` by `#[path]`/plain `mod`, in order.

    Missing files are dropped here rather than raised: this module's callers are
    gates whose subject is the *text* of the source, and
    `audit-case-migration.py` already fails hard (exit 2) on a `mod` naming a
    file that does not exist, so a second, differently-worded hard failure in
    five more places buys nothing.

    `base` separates WHERE THE TEXT COMES FROM (`rs_path`) from WHERE ITS `mod`
    declarations RESOLVE (`base`, defaulting to `rs_path`). `batch5_crosscheck.py`
    needs that split: a `--pretrim`/`=PATH` override hands it a git blob written
    to a temp file, whose `#[path = "sub/leaf.rs"]` would otherwise resolve under
    `/tmp` and silently yield NO submodules -- turning every qualified citation
    in a `#[path]` retention pair into a spurious "names a file that is not a
    submodule". The blob is a copy of a tree file, so the tree file's directory
    is the right base for it (fix round 1, I5).
    """
    mod = mod or audit_module()
    path = Path(rs_path)
    paths = mod.resolve_path_mods(Path(base) if base else path, path.read_text())
    return [p for p in paths if p.is_file()]


def read_with_submodules(rs_path, mod=None):
    """`rs_path`'s text with every reachable submodule's text appended.

    Joined with a newline, in the same order and by the same rule
    `audit-case-migration.py`'s `main` uses to build `old_source_combined`, so a
    literal-coverage gate reading this sees exactly the corpus the audit does.
    """
    path = Path(rs_path)
    pieces = [path.read_text()]
    pieces.extend(p.read_text() for p in submodule_paths(path, mod))
    return "\n".join(pieces)
