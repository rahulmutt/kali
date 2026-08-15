#!/usr/bin/env python3
r"""Rebuild the anchor stratum of the frozen blast-radius corpus from its sources.

WHAT THIS IS FOR
    The published ranking carries a `corpus_hash` (see
    `tools/blast-radius/corpus/manifest.json`). A reader holding that hash must
    be able to reconstruct what was measured and check it against the tree,
    which is what this script and its companion
    `verify_anchor_extraction.py` exist to make possible.

WHERE THE 137 ANCHOR PROGRAMS COME FROM
    6   The CLBG benchmarks, vendored as plain-JS fixture files at
        crates/kali_cli/tests/fixtures/benchmarks/<name>-benchmark-v1.ts.
        They carry a `.ts` extension but hold no type annotations. Note that
        the design spec's original §4.1 claim that these live as inline Rust
        string literals is wrong and is corrected by its 2026-08-15 (Task 11)
        amendment: only binary-trees has a literal, and that one is a
        parameterised `format!` template (n=10) rather than a fixed program,
        so the vendored n=21 fixture the acceptance gate runs is what is
        extracted here.
    131 The programs held as inline Rust string literals in
        crates/kali_cli/tests/imperative_core_runtime.rs -- 118 passed to
        `run_js` and 13 to `run_js_expect_failure`. The rejected ones are in
        the corpus deliberately: excluding them would be curation by
        acceptance, which design spec §4.2 forbids. Acceptance is measured
        per program by the counter, as a separate and separately-reported
        step.

    They are ordinary escaped `"..."` literals, not `r#"..."#`, and the file
    leans on Rust's `\<newline>` line-continuation, so they cannot be lifted
    with a regex. This module lexes the Rust source properly (line and block
    comments, raw strings, char literals versus lifetimes, `\x`/`\u{}`
    escapes, line continuations) and resolves `let source = "...";`
    indirection before a `run_js(source)` call.

NAMING
    CLBG:            anchor/clbg_<benchmark>.js, after the
                     crates/kali_cli/tests/clbg_*_runtime.rs file stems.
    imperative_core: anchor/<test_fn>.js when the holding test function has one
                     program, anchor/<test_fn>_NN.js (source order) when it has
                     several. 55 test functions hold the 131 programs, so the
                     one-file-per-test-function rule cannot stand alone.

RE-RUN
    python3 tools/blast-radius/extract_anchor_corpus.py --check
        Rebuild into a temporary directory and diff against the committed
        corpus. Exits non-zero on any difference. Changes nothing.

    python3 tools/blast-radius/extract_anchor_corpus.py --out tools/blast-radius/corpus/anchor
        Overwrite the committed corpus. THIS BREAKS THE FREEZE unless the
        manifest is regenerated with it -- see the generator in
        `.superpowers/sdd/2026-08-15-blast-radius-ranking/task-11-brief.md`
        Step 5 -- and a corpus_hash change invalidates any ranking already
        published against the old one (design spec §4.3).

    Run from the repository root. Importing this module has no side effects.
"""

import argparse
import filecmp
import json
import pathlib
import re
import shutil
import sys
import tempfile

IMPERATIVE_SOURCE = pathlib.Path("crates/kali_cli/tests/imperative_core_runtime.rs")
BENCHMARK_DIR = pathlib.Path("crates/kali_cli/tests/fixtures/benchmarks")
COMMITTED_ANCHOR = pathlib.Path("tools/blast-radius/corpus/anchor")
PROVENANCE = pathlib.Path("tools/blast-radius/anchor-provenance.json")

# CLBG fixture stem -> corpus file stem. The fixtures are the programs the
# `crates/kali_cli/tests/clbg_*_runtime.rs` acceptance gates actually run.
CLBG = {
    "binary-trees-benchmark-v1.ts": "clbg_binary_trees.js",
    "fannkuch-redux-benchmark-v1.ts": "clbg_fannkuch.js",
    "fasta-benchmark-v1.ts": "clbg_fasta.js",
    "mandelbrot-benchmark-v1.ts": "clbg_mandelbrot.js",
    "nbody-benchmark-v1.ts": "clbg_nbody.js",
    "spectral-norm-benchmark-v1.ts": "clbg_spectral_norm.js",
}


class Tok:
    __slots__ = ("kind", "text", "value", "start", "end", "line")

    def __init__(self, kind, text, value, start, end, line):
        self.kind = kind
        self.text = text  # the verbatim source span, delimiters included
        self.value = value  # decoded contents, for string tokens
        self.start = start
        self.end = end
        self.line = line

    def __repr__(self):
        return f"Tok({self.kind},{self.text!r})"


def unescape_rust(body):
    """Decode the *contents* of a non-raw Rust string literal."""
    out = []
    i = 0
    m = len(body)
    while i < m:
        c = body[i]
        if c != "\\":
            out.append(c)
            i += 1
            continue
        i += 1
        e = body[i]
        if e == "n":
            out.append("\n")
            i += 1
        elif e == "t":
            out.append("\t")
            i += 1
        elif e == "r":
            out.append("\r")
            i += 1
        elif e == "0":
            out.append("\0")
            i += 1
        elif e in ("\\", "'", '"'):
            out.append(e)
            i += 1
        elif e == "x":
            out.append(chr(int(body[i + 1 : i + 3], 16)))
            i += 3
        elif e == "u":
            assert body[i + 1] == "{", body[i : i + 8]
            j = body.index("}", i)
            out.append(chr(int(body[i + 2 : j], 16)))
            i = j + 1
        elif e == "\n":
            # Line continuation: the newline and the following indentation are
            # not part of the string. This file uses it heavily.
            i += 1
            while i < m and body[i] in " \t\r\n":
                i += 1
        else:
            raise ValueError(f"unknown escape \\{e}")
    return "".join(out)


def tokenize(text):
    toks = []
    i = 0
    n = len(text)
    line = 1
    while i < n:
        c = text[i]
        if c == "\n":
            line += 1
            i += 1
            continue
        if c in " \t\r":
            i += 1
            continue
        if text.startswith("//", i):
            j = text.find("\n", i)
            i = n if j < 0 else j
            continue
        if text.startswith("/*", i):
            depth = 1
            i += 2
            while i < n and depth:
                if text.startswith("/*", i):
                    depth += 1
                    i += 2
                elif text.startswith("*/", i):
                    depth -= 1
                    i += 2
                else:
                    line += text[i] == "\n"
                    i += 1
            continue
        m = re.match(r'(b?)r(#*)"', text[i:])
        if m and (i == 0 or not (text[i - 1].isalnum() or text[i - 1] == "_")):
            hashes = m.group(2)
            start = i
            body_start = i + m.end()
            close = '"' + hashes
            j = text.index(close, body_start)
            body = text[body_start:j]
            toks.append(
                Tok("str", text[start : j + len(close)], body, start, j + len(close), line)
            )
            line += text.count("\n", start, j)
            i = j + len(close)
            continue
        if c == '"':
            start = i
            i += 1
            while True:
                if text[i] == "\\":
                    i += 2
                elif text[i] == '"':
                    i += 1
                    break
                else:
                    i += 1
            body = text[start + 1 : i - 1]
            toks.append(Tok("str", text[start:i], unescape_rust(body), start, i, line))
            line += text.count("\n", start, i)
            continue
        if c == "'":
            m = re.match(r"'(\\.[^']*|[^\\'])'", text[i:])
            if m:  # char literal
                toks.append(Tok("char", m.group(0), None, i, i + m.end(), line))
            else:  # lifetime
                m = re.match(r"'[A-Za-z_][A-Za-z0-9_]*", text[i:])
                toks.append(Tok("lifetime", m.group(0), None, i, i + m.end(), line))
            i += m.end()
            continue
        m = re.match(r"[A-Za-z_][A-Za-z0-9_]*", text[i:])
        if m:
            toks.append(Tok("ident", m.group(0), None, i, i + m.end(), line))
            i += m.end()
            continue
        toks.append(Tok("punct", c, None, i, i + 1, line))
        i += 1
    return toks


def call_sites(text):
    """Every (test_fn, helper, literal token) in source order.

    Raises if any `run_js*` argument is neither a literal nor a `let`-bound
    literal: a silently-skipped call site would mean a program missing from the
    frozen corpus, which is the one failure mode this must not have.
    """
    toks = tokenize(text)
    found = []
    cur_fn = None
    depth = 0
    bindings = {}
    unresolved = []
    for idx, t in enumerate(toks):
        if t.kind == "punct" and t.text == "{":
            depth += 1
        elif t.kind == "punct" and t.text == "}":
            depth -= 1
            if depth == 0:
                cur_fn, bindings = None, {}
        if t.kind == "ident" and t.text == "fn" and depth == 0:
            cur_fn, bindings = toks[idx + 1].text, {}
        if (
            t.kind == "ident"
            and t.text == "let"
            and toks[idx + 1].kind == "ident"
            and toks[idx + 2].text == "="
            and toks[idx + 3].kind == "str"
            and toks[idx + 4].text == ";"
        ):
            bindings[toks[idx + 1].text] = toks[idx + 3]
        if t.kind == "ident" and t.text in ("run_js", "run_js_expect_failure"):
            if toks[idx - 1].kind == "ident" and toks[idx - 1].text == "fn":
                continue  # the helper's own definition
            assert toks[idx + 1].text == "(", (t.line, toks[idx + 1])
            arg = toks[idx + 2]
            if arg.kind == "punct" and arg.text == "&":
                arg = toks[idx + 3]
            if arg.kind == "str":
                found.append((cur_fn, t.text, arg))
            elif arg.kind == "ident" and arg.text in bindings:
                found.append((cur_fn, t.text, bindings[arg.text]))
            else:
                unresolved.append((cur_fn, t.line, arg))
    if unresolved:
        raise SystemExit(
            "unresolved run_js call sites (a program would be missing from the "
            f"corpus): {unresolved}"
        )
    return found


def anchor_programs():
    """[(filename, bytes, provenance dict)] for all 137 anchor programs."""
    out = []

    for fixture, name in sorted(CLBG.items(), key=lambda kv: kv[1]):
        path = BENCHMARK_DIR / fixture
        out.append(
            (
                name,
                path.read_bytes(),
                {
                    "file": name,
                    "origin": "clbg-fixture",
                    "source": path.as_posix(),
                    "test_fn": None,
                    "line": None,
                    "helper": None,
                },
            )
        )

    sites = call_sites(IMPERATIVE_SOURCE.read_text(encoding="utf-8"))
    totals = {}
    for fn, _, _ in sites:
        totals[fn] = totals.get(fn, 0) + 1
    seen = {}
    for fn, helper, tok in sites:
        seen[fn] = seen.get(fn, 0) + 1
        name = f"{fn}.js" if totals[fn] == 1 else f"{fn}_{seen[fn]:02d}.js"
        out.append(
            (
                name,
                tok.value.encode("utf-8"),
                {
                    "file": name,
                    "origin": "inline-rust-literal",
                    "source": IMPERATIVE_SOURCE.as_posix(),
                    "test_fn": fn,
                    "line": tok.line,
                    "helper": helper,
                },
            )
        )
    return out


def write(programs, out_dir):
    out_dir = pathlib.Path(out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    for name, body, _ in programs:
        (out_dir / name).write_bytes(body)


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--out", help="directory to write the anchor programs into")
    group.add_argument(
        "--check",
        action="store_true",
        help="rebuild into a temp dir and diff against the committed corpus",
    )
    parser.add_argument(
        "--write-provenance",
        action="store_true",
        help=f"also refresh {PROVENANCE}",
    )
    args = parser.parse_args()

    programs = anchor_programs()
    names = [name for name, _, _ in programs]
    if len(set(names)) != len(names):
        raise SystemExit("two programs claimed the same filename")

    if args.write_provenance:
        PROVENANCE.write_text(
            json.dumps([p for _, _, p in sorted(programs)], indent=2) + "\n"
        )

    if args.out:
        write(programs, args.out)
        print(f"wrote {len(programs)} anchor programs to {args.out}")
        return 0

    tmp = tempfile.mkdtemp(prefix="blast-radius-anchor-")
    try:
        write(programs, tmp)
        match, mismatch, errors = filecmp.cmpfiles(
            tmp, COMMITTED_ANCHOR, names, shallow=False
        )
        on_disk = sorted(p.name for p in COMMITTED_ANCHOR.iterdir())
        extra = sorted(set(on_disk) - set(names))
        if mismatch or errors or extra:
            print(f"MISMATCH   differ: {mismatch}", file=sys.stderr)
            print(f"           errors: {errors}", file=sys.stderr)
            print(f"           on disk but not re-derived: {extra}", file=sys.stderr)
            return 1
        print(f"{len(match)} anchor programs re-derived, all byte-identical to the corpus")
        return 0
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


if __name__ == "__main__":
    sys.exit(main())
