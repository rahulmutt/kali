#!/usr/bin/env python3
"""Write `t19b5_captures.py` from a capture directory, by script, never by hand.

`t19b5_capture_run.py` produces the byte-exact outputs; this turns them into a
Python module and ASSERTS THE ROUND TRIP before writing, so no byte of the
result passes through a transcription step. Committed for the same reason the
capture runner is (U12): the constants have to be re-derivable, not trusted.

  Usage:  t19b5_write_captures.py <capture-dir>
"""

from __future__ import annotations

import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))

HEADER = '''"""Rule-8 / rule-9 captured fixture texts for Task 19 batch 5.

EVERY CONSTANT HERE IS THE BYTE-EXACT OUTPUT OF EXECUTING THE REAL CODE. Rule 8
forbids hand-simulating a `format!`; rule 9 extends the same discipline to a
fixture built one level removed inside a library crate (`kali_common::`). The
fixtures in this batch that exist as plain string literals are NOT here --
`gen_task19_batch5.py` pulls those straight out of the `.rs` through
`lexer.find_string_literals`, which is this project's copy-never-retype
mechanism. Only a fixture that exists as a literal NOWHERE needs a capture.

HOW THEY WERE CAPTURED, so they can be re-derived -- `t19b5_capture_run.py`,
committed beside this file, is the script and it uses two mechanisms:

  A. `RUNTIME_FORIN` -- the eleven `runtime_forin` `#[test]`s whose fixture is
     built by the `format!` inside `forin_ternary_case` / `forin_leak_case`.
     Those helpers hand the string straight to `run_source`, so there is no
     builder to call; but `run_source` writes the program to
     `std::env::temp_dir()/kali-forin-<pid>-<counter>-<len>/main.ts` and never
     cleans up. Each `#[test]` was run ALONE with its own `TMPDIR`, and the one
     file left behind is the real `format!`'s output, obtained by executing it.

  B. `SPREAD` -- `for_of_array_iteration_spread`'s five builder outputs, taken
     by `include!`ing the shipped source into a module and calling the builders
     from inside it (they are private, so a sibling module cannot name them).
     `include!` rather than a copy, so the executed `format!` and the executed
     `kali_common::` call are literally the ones in the source under migration.

WHAT KEEPS THEM FROM GOING STALE. `gen_task19_batch5.check_captured` re-checks
every capture against its own `.rs` before it is emitted: the fn must still
exist, and it must still build its value the way the capture was taken for. A
capture taken before a source edit therefore fails the generator instead of
shipping a program that is no longer the program under test. The eleven
`RUNTIME_FORIN` entries carry a second, independent check -- the generator
recomputes the `format!` substitution structurally and requires it to agree
with the executed bytes. The capture is what SHIPS; the recomputation is a
cross-check that would catch either one being wrong.
"""

'''


def render(name: str, value: str) -> str:
    lines = value.split("\n")
    out = [f"{name} = ("]
    for i, ln in enumerate(lines):
        piece = ln + ("\n" if i < len(lines) - 1 else "")
        if piece == "":
            continue
        out.append("    " + repr(piece))
    out.append(")")
    return "\n".join(out)


def main(argv):
    if not argv:
        raise SystemExit(__doc__)
    d = os.path.abspath(argv[0])
    forin, spread = {}, {}
    for f in sorted(os.listdir(d)):
        if not f.endswith(".txt"):
            continue
        body = open(os.path.join(d, f), encoding="utf-8").read()
        if f.startswith("forin__"):
            forin[f[len("forin__"):-len(".txt")]] = body
        elif f.startswith("spread__"):
            spread[f[len("spread__"):-len(".txt")]] = body
    if not forin or not spread:
        raise SystemExit("capture directory is missing one of the two families")

    chunks = [HEADER]
    chunks.append("# key: the `#[test]` fn whose single invocation wrote this "
                  "program.\nRUNTIME_FORIN = {\n")
    for k, v in sorted(forin.items()):
        chunks.append(f"    {k!r}:\n" + _indent(render("_", v), 8)
                      .replace("_ = (", "(", 1) + ",\n")
    chunks.append("}\n\n")
    chunks.append("# key: the builder call, spelled as the evaluator renders "
                  "it.\nSPREAD = {\n")
    names = {
        "array_from_iteration_body": "array_from_iteration_body()",
        "browser_harness_array_from_source__run":
            'browser_harness_array_from_source("run")',
        "browser_harness_array_from_source__test":
            'browser_harness_array_from_source("test")',
        "set_map_break_continue__run":
            'browser_harness_array_from_set_map_break_continue_source("run")',
        "set_map_break_continue__test":
            'browser_harness_array_from_set_map_break_continue_source("test")',
    }
    for k, v in sorted(spread.items()):
        if k not in names:
            raise SystemExit(f"unexpected capture file `{k}`")
        chunks.append(f"    {names[k]!r}:\n" + _indent(render("_", v), 8)
                      .replace("_ = (", "(", 1) + ",\n")
    chunks.append("}\n")
    text = "".join(chunks)

    out = os.path.join(HERE, "t19b5_captures.py")
    with open(out, "w", encoding="utf-8") as f:
        f.write(text)

    # THE ROUND TRIP IS ASSERTED, not assumed: import what was just written and
    # require every value to equal the bytes on disk.
    sys.path.insert(0, HERE)
    import importlib
    mod = importlib.import_module("t19b5_captures")
    importlib.reload(mod)
    for k, v in forin.items():
        if mod.RUNTIME_FORIN[k] != v:
            raise SystemExit(f"round trip failed for RUNTIME_FORIN[{k!r}]")
    for k, v in spread.items():
        if mod.SPREAD[names[k]] != v:
            raise SystemExit(f"round trip failed for SPREAD[{names[k]!r}]")
    print(f"WROTE {out}: {len(forin)} runtime_forin, {len(spread)} spread; "
          f"round trip asserted")
    return 0


def _indent(s: str, n: int) -> str:
    return "\n".join((" " * n) + ln if ln.strip() else ln for ln in s.split("\n"))


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
