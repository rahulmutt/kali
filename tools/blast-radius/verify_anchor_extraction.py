#!/usr/bin/env python3
"""Audit the frozen anchor corpus against its sources using rustc as the decoder.

WHY A SECOND SCRIPT
    `extract_anchor_corpus.py --check` re-derives the corpus, but with the same
    Rust-literal decoder that produced it: a bug in that decoder would agree
    with itself. This script is the independent check. It emits a Rust program
    whose string literals are the *verbatim source spans* spliced out of
    `crates/kali_cli/tests/imperative_core_runtime.rs` -- no re-escaping by
    Python -- compiles it with rustc, and lets Rust's own lexer produce the
    bytes. Those bytes are then compared to the committed corpus file by file.

    Passing means every inline program in the corpus is character-for-character
    what the Rust compiler reads from the test file. That is the strongest
    statement available short of shipping the corpus as `include_str!`.

    The six CLBG programs need no decoder: they are vendored files, compared
    directly.

RE-RUN
    python3 tools/blast-radius/verify_anchor_extraction.py

    Run from the repository root. Requires `rustc` on PATH. Exits non-zero on
    any difference, and changes nothing. Pass `--keep` to leave the generated
    Rust source and the decoded files behind for inspection.
"""

import argparse
import filecmp
import importlib.util
import pathlib
import shutil
import subprocess
import sys
import tempfile

HERE = pathlib.Path(__file__).resolve().parent
COMMITTED_ANCHOR = pathlib.Path("tools/blast-radius/corpus/anchor")


def load_extractor():
    spec = importlib.util.spec_from_file_location(
        "extract_anchor_corpus", HERE / "extract_anchor_corpus.py"
    )
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--keep", action="store_true", help="keep the scratch directory")
    args = parser.parse_args()

    extractor = load_extractor()
    if not COMMITTED_ANCHOR.is_dir():
        raise SystemExit(f"{COMMITTED_ANCHOR} not found -- run from the repository root")

    # Re-derive the file names exactly as the extractor does, but keep the raw
    # literal *span* rather than its decoded value.
    sites = extractor.call_sites(
        extractor.IMPERATIVE_SOURCE.read_text(encoding="utf-8")
    )
    totals = {}
    for fn, _, _ in sites:
        totals[fn] = totals.get(fn, 0) + 1
    seen = {}
    entries = []
    for fn, _, tok in sites:
        seen[fn] = seen.get(fn, 0) + 1
        name = f"{fn}.js" if totals[fn] == 1 else f"{fn}_{seen[fn]:02d}.js"
        # tok.text is the source span verbatim, delimiters and all, so rustc
        # sees byte-for-byte what the test file holds.
        entries.append(f'    ("{name}", {tok.text}),\n')

    scratch = pathlib.Path(tempfile.mkdtemp(prefix="blast-radius-verify-"))
    try:
        source = scratch / "verifier.rs"
        source.write_text(
            "fn main() {\n"
            "    let out = std::env::args().nth(1).unwrap();\n"
            "    let items: &[(&str, &str)] = &[\n"
            + "".join(entries)
            + "    ];\n"
            "    for (name, body) in items {\n"
            '        std::fs::write(format!("{out}/{name}"), body).unwrap();\n'
            "    }\n"
            "}\n"
        )
        binary = scratch / "verifier"
        subprocess.run(
            ["rustc", "-O", "-o", str(binary), str(source)], check=True
        )
        decoded = scratch / "decoded"
        decoded.mkdir()
        subprocess.run([str(binary), str(decoded)], check=True)

        names = [line.split('"')[1] for line in entries]
        match, mismatch, errors = filecmp.cmpfiles(
            decoded, COMMITTED_ANCHOR, names, shallow=False
        )
        failed = False
        if mismatch or errors:
            print(f"MISMATCH  differ: {mismatch}  errors: {errors}", file=sys.stderr)
            failed = True
        else:
            print(
                f"{len(match)}/{len(names)} inline programs are byte-identical to the "
                "literals as rustc decodes them"
            )

        for fixture, name in sorted(extractor.CLBG.items(), key=lambda kv: kv[1]):
            left = extractor.BENCHMARK_DIR / fixture
            right = COMMITTED_ANCHOR / name
            if not filecmp.cmp(left, right, shallow=False):
                print(f"MISMATCH  {name} differs from {left}", file=sys.stderr)
                failed = True
        if not failed:
            print(f"{len(extractor.CLBG)}/6 CLBG programs are byte-identical to their fixtures")

        on_disk = sorted(p.name for p in COMMITTED_ANCHOR.iterdir())
        expected = sorted(names + list(extractor.CLBG.values()))
        if on_disk != expected:
            print(
                "MISMATCH  corpus directory holds files no source accounts for: "
                f"{sorted(set(on_disk) - set(expected))}",
                file=sys.stderr,
            )
            failed = True
        return 1 if failed else 0
    finally:
        if args.keep:
            print(f"scratch kept at {scratch}")
        else:
            shutil.rmtree(scratch, ignore_errors=True)


if __name__ == "__main__":
    sys.exit(main())
