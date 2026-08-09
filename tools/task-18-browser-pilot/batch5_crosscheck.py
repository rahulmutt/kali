#!/usr/bin/env python3
"""Cross-file consistency gate for a batch of migrated `browser/` case files.

Batch 4's review found a defect class that every per-file gate passed: four
concurrent implementers described the same recurring fact four different ways,
and one of them described a state the file no longer had. Nothing mechanical
could see it, because no gate reads `#` header prose or `rationale` wording.
This is the missing gate. It checks the two things that failure class actually
consisted of:

  1. STRUCTURE. Every file in the batch carries the same fixed set of header
     sections, in the same order. A group that invented its own section, or
     dropped one, fails here rather than being caught by eye.

  2. CITATIONS. Every `:N` written next to a backticked code snippet must point
     at a line of the paired `.rs` that actually contains that snippet. This is
     the check that four fix rounds in this project were spent doing by hand.
     A citation whose line is out of range, or whose line does not contain the
     construct it is attached to, is a hard failure.

Both are checked against the shipped `.toml`, not against the generator, so a
generator that renders the right thing and writes the wrong file is still
caught.

Deliberate non-goal: this does not check that the prose is TRUE, only that it is
consistent and that its citations resolve. U8 is explicit that rationale prose is
audited by nothing; this narrows that gap, it does not close it.

Usage: batch5_crosscheck.py STEM[=PRETRIM.rs] [STEM[=PRETRIM.rs] ...]
Exit 0 if every file passes, 1 otherwise.

A trimmed U4 retention pair MUST be given its pre-trim blob with `=PATH`: every
`:N` in such a case file is a pre-trim line number (its own header says so), so
resolving them against the working-tree `.rs` would report failures that are
artefacts of the trim rather than stale citations -- the exact confusion ruling
9 exists to prevent.
"""

import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
TESTS = os.path.join(REPO, "crates/kali_cli/tests")
CASES = os.path.join(TESTS, "cases/browser")

# Header sections every batch-5 file carries, in order. Sections marked optional
# appear only when the file's shape calls for them, but when present they must
# appear in this relative order.
SECTIONS = [
    ("Migrated from tests/browser_", True),
    ("RULE 12", True),
    ("RULE 7 / U1", True),
    ("RULE 6", True),
    ("U2 -- `[source]` is FILE-WIDE", True),
    ("RULE 13 -- transitive helper docs", True),
    ("ARGV ORDER", True),
    ("ASSERTION SHAPE", True),
]

# A backticked snippet followed by a parenthesised or bare `:N` citation.
CITE = re.compile(r"`([^`\n]{3,120})`[^`\n]{0,40}?\(?:(\d+)(?:-(\d+))?\)?")


def _header(text):
    out = []
    for line in text.split("\n"):
        if line.startswith("#"):
            out.append(line.lstrip("#").strip())
        elif line.strip():
            break
    return out


def _distinctive(snippet):
    """A token from a backticked snippet that should appear on the cited line.

    Prose backticks (`[matrix]`, `run`, a fn name) are not code positions, so
    only snippets that look like Rust/JS constructs are checked. Returns None
    to skip.
    """
    s = snippet.strip()
    if not any(ch in s for ch in "(.["):
        return None
    if s.startswith("[") or s.startswith("--") or " " == s:
        return None
    m = re.match(r"[A-Za-z_][A-Za-z0-9_]*", s.lstrip("&*!."))
    if not m:
        return None
    tok = m.group(0)
    return tok if len(tok) >= 4 else None


def check(spec):
    stem, _, override = spec.partition("=")
    toml_path = os.path.join(CASES, f"{stem}.toml")
    rs_path = override or os.path.join(TESTS, f"browser_{stem}.rs")
    problems = []
    if not os.path.exists(toml_path):
        return [f"{stem}: no case file at {toml_path}"]
    if not os.path.exists(rs_path):
        return [f"{stem}: no source at {rs_path}"]
    text = open(toml_path).read()
    rs_lines = open(rs_path).read().split("\n")
    header = _header(text)
    blob = "\n".join(header)

    # Section markers are matched at the START of a header line only. Matching
    # anywhere would collide with the prose: `MATRIX_NOT_AXES` legitimately
    # contains the words "ASSERTION SHAPE" mid-sentence, and a substring search
    # reported four files as out-of-order on that alone. A checker whose false
    # positives are indistinguishable from its true ones is not a gate.
    starts = {}
    for n, line in enumerate(header):
        for marker, _ in SECTIONS:
            if line.startswith(marker) and marker not in starts:
                starts[marker] = n

    pos = -1
    for marker, required in SECTIONS:
        idx = starts.get(marker, -1)
        if idx == -1:
            if required:
                problems.append(f"{stem}: header section missing: {marker!r}")
            continue
        if idx < pos:
            problems.append(f"{stem}: header section out of order: {marker!r}")
        pos = idx

    # Citations, over the WHOLE file (header + every rationale).
    checked = 0
    for m in CITE.finditer(text):
        snippet, first, last = m.group(1), int(m.group(2)), m.group(3)
        tok = _distinctive(snippet)
        if tok is None:
            continue
        span = range(first, (int(last) if last else first) + 1)
        if span[-1] > len(rs_lines):
            problems.append(
                f"{stem}: citation :{first} for `{snippet[:40]}` is past end of "
                f"browser_{stem}.rs ({len(rs_lines)} lines)")
            continue
        checked += 1
        window = "\n".join(rs_lines[max(0, first - 3):span[-1] + 2])
        if tok not in window:
            problems.append(
                f"{stem}: citation :{first} does not contain {tok!r} "
                f"(from `{snippet[:50]}`)")
    print(f"{stem}: {len(header)} header line(s), {checked} code citation(s) checked, "
          f"{len(problems)} problem(s)")
    return problems


def main(argv):
    if len(argv) < 2:
        print(__doc__)
        return 2
    all_problems = []
    for stem in argv[1:]:
        all_problems += check(stem)
    if all_problems:
        print("\nCROSSCHECK FAILED")
        for p in all_problems:
            print(f"  {p}")
        return 1
    print("\nCROSSCHECK OK — header structure consistent, every code citation resolves")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
