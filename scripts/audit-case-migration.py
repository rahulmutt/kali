#!/usr/bin/env python3
"""Fail if a migrated case file drops a claim its .rs predecessor made.

Migrating ~200k lines of assertions is where meaning gets silently dropped, and
this repository has already had two fail-closed tests degrade to asserting
nothing. So the migration gate is mechanical, not eyeballed: every string
literal the old test compared against, every JSON path it asserted on, and every
argv token it passed must still appear somewhere in the new case files.

Five claim kinds are extracted:
  - `.contains("literal")` string arguments.
  - `const NAME: &str = "literal";` rule constants.
  - `assert_eq!(lhs, "literal")` right-hand-side string values. Measured across
    this crate's tests/*.rs, assert_eq! string-literal comparisons (1,586
    sites) outnumber .contains() literals (1,134 sites) by a wide margin --
    this is the dominant assertion form, not .contains(). A migration that
    keeps a JSON path (`errors.0.code`) but silently asserts a different value
    ("E5507" instead of "E5506") is exactly the kind of quiet weakening this
    script exists to catch, and only this claim kind catches it.
  - Bracketed JSON keys inside an indexing expression, e.g. the `code` in
    `json["errors"][0]["code"]`.
  - `.arg("token")` argv tokens.

Every string-literal claim (contains/const/assert_eq) is checked against the
new case files in *two* spellings: the literal exactly as written in the Rust
source (escapes intact, e.g. `a\nb`), and the fully-unescaped text (e.g. a
real embedded newline). A TOML case file may carry a newline-bearing value
either as a basic string with a `\n` escape (matches the first spelling) or as
a `'''...'''` literal block with a real newline (matches the second). A claim
counts as present if *either* spelling appears anywhere in the new text.

This is a coverage check, not a proof of equivalence. It catches wholesale drops
and quiet weakenings (a rule constant vanishing while `contains("E5506")`
survives). It cannot catch a claim that was rewritten to be weaker while keeping
its literals. Read the diff too.

Usage: audit-case-migration.py OLD.rs NEW.toml [NEW.toml ...]
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

# Literal arguments to .contains(...) — one of several dominant assertion forms.
CONTAINS = re.compile(r'\.contains\(\s*(?:&)?(r?#*"(?:[^"\\]|\\.)*"#*)')
# const NAME: &str = "literal";
CONST = re.compile(r'const\s+[A-Z0-9_]+\s*:\s*&str\s*=\s*\n?\s*(r?#*"(?:[^"\\]|\\.)*"#*)')
# assert_eq!(lhs, "literal") — the dominant assertion form by site count.
# The first argument is captured with a bare [^,]* (no comma allowed) rather
# than a real expression parser: this is deliberately conservative. It correctly
# skips calls whose first argument itself contains a top-level comma (a nested
# multi-arg call, a vec! literal, ...) rather than risk mis-splitting them, at
# the cost of a false negative for that (rare, and always non-string-adjacent
# in this corpus at time of writing) shape. It matches across newlines, so
# multi-line assert_eq!(...) calls are still captured.
ASSERT_EQ = re.compile(r'assert_eq!\(\s*[^,]*,\s*(r?#*"(?:[^"\\]|\\.)*"#*)\s*[,)]')
# assert_eq!(json["a"]["b"], value) — capture each bracketed key.
JSON_KEY = re.compile(r'\[\s*"([A-Za-z_][A-Za-z0-9_]*)"\s*\]')
# .arg("token")
ARG = re.compile(r'\.arg\(\s*"([^"]*)"\s*\)')
TEST_FN = re.compile(r'#\[test\][^\n]*\n(?:\s*#\[[^\]]*\]\s*\n)*\s*(?:async\s+)?fn\s+([a-z0-9_]+)')

# String-literal claim kinds, each checked in both spellings (see module docstring).
LITERAL_KINDS: dict[str, re.Pattern[str]] = {
    "contains literals": CONTAINS,
    "rule constants": CONST,
    "assert_eq values": ASSERT_EQ,
}

# Per-kind values with no discriminating power, excluded so they can't produce
# a false AUDIT OK. json keys and argv tokens keep their own literal text as
# both canonical form and sole search variant, so no unquoting applies there.
BORING: dict[str, set[str]] = {
    # Trivially-common argv tokens.
    "argv tokens": {"run", "check", "build", "test", "json", "--output"},
    # "0" and "1" appear as substrings of case/schema numbers, ports, exit
    # codes, etc. in essentially every case file, so a substring check for
    # them never actually discriminates a dropped claim from a present one.
    # (A bare "" is excluded everywhere below: it is a substring of every
    # string, so checking for it is a permanent no-op regardless of kind.)
    "assert_eq values": {"0", "1"},
}


def unquote(raw: str) -> str:
    """Turn a Rust string literal token into its fully-unescaped text."""
    raw = raw.strip()
    if raw.startswith("r"):
        raw = raw[1:]
        hashes = len(raw) - len(raw.lstrip("#"))
        return raw[hashes + 1 : len(raw) - hashes - 1]
    body = raw[1:-1]
    return (
        body.replace('\\"', '"')
        .replace("\\n", "\n")
        .replace("\\t", "\t")
        .replace("\\\\", "\\")
    )


def raw_body(raw: str) -> str:
    """The literal text between a Rust string literal's quotes, escapes left
    exactly as written (e.g. a two-character `\\n`, not a real newline).

    Raw strings (r"..."/r#"..."#) have no escapes to leave intact, so this
    collapses to unquote() for them.
    """
    raw = raw.strip()
    if raw.startswith("r"):
        return unquote(raw)
    return raw[1:-1]


def literal_variants(token: str) -> frozenset[str]:
    """Both spellings a Rust string literal's contents might take in a TOML
    case file: as written (escapes intact) and fully unescaped. See the
    module docstring for why both must be checked."""
    return frozenset({raw_body(token), unquote(token)})


def claims(source: str) -> dict[str, dict[str, frozenset[str]]]:
    """kind -> {canonical display value -> spellings to search for}."""
    out: dict[str, dict[str, frozenset[str]]] = {kind: {} for kind in LITERAL_KINDS}
    out["json keys"] = {}
    out["argv tokens"] = {}

    for kind, pattern in LITERAL_KINDS.items():
        for token in pattern.findall(source):
            canonical = unquote(token)
            bucket = out[kind]
            bucket[canonical] = bucket.get(canonical, frozenset()) | literal_variants(token)

    for key in JSON_KEY.findall(source):
        out["json keys"][key] = frozenset({key})

    for tok in ARG.findall(source):
        out["argv tokens"][tok] = frozenset({tok})

    return out


def main() -> int:
    if len(sys.argv) < 3:
        print(__doc__, file=sys.stderr)
        return 2

    old_path = Path(sys.argv[1])
    new_paths = [Path(p) for p in sys.argv[2:]]

    old_source = old_path.read_text()
    new_text = "\n".join(p.read_text() for p in new_paths)

    old_claims = claims(old_source)

    missing: list[tuple[str, str]] = []
    for kind, entries in old_claims.items():
        exclude = BORING.get(kind, set())
        for canonical, variants in sorted(entries.items()):
            if not canonical or canonical in exclude:
                continue
            if not any(variant and variant in new_text for variant in variants):
                missing.append((kind, canonical))

    old_tests = sorted(set(TEST_FN.findall(old_source)))
    print(f"{old_path}: {len(old_tests)} #[test] fns")
    for kind, entries in old_claims.items():
        print(f"  {kind}: {len(entries)}")

    if missing:
        print(f"\nAUDIT FAILED — {len(missing)} claim(s) absent from the case files:")
        for kind, value in missing:
            print(f"  [{kind}] {value!r}")
        return 1

    print("\nAUDIT OK — every literal claim is present in the case files.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
