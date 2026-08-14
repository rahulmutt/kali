#!/usr/bin/env python3
"""Take the rule-8 / rule-9 captures for Task 19 batch 5, by EXECUTING the code.

Rule 8 forbids hand-simulating a `format!`; rule 9 extends the same discipline
to a fixture built one level removed inside a library crate (`kali_common::`).
Neither fixture class exists as a literal anywhere in the source, so neither can
be copied -- the only compliant route is to run the real code and take its
bytes. This script is that route, and it is committed (U12) so the constants in
`t19b5_captures.py` can be re-derived rather than trusted.

TWO MECHANISMS, because the two sources hide their `format!` in different places.

  A. `runtime_forin.rs` -- RUN THE REAL `#[test]`, THEN READ WHAT IT WROTE.
     Its `format!`s live inside `forin_ternary_case` / `forin_leak_case`, which
     immediately hand the string to `run_source`, so there is no builder to call
     and no return value to print. But `run_source` writes the program to
     `std::env::temp_dir()/kali-forin-<pid>-<counter>-<len>/main.ts` and never
     cleans up. So: run ONE `#[test]` at a time with its own `TMPDIR`, and the
     single file left behind IS the byte-exact output of the real `format!`,
     obtained by executing it. No expression is retyped and nothing is
     simulated.

  B. `for_of_array_iteration_spread.rs` -- `include!` THE SOURCE AND CALL THE
     BUILDER. Its fixtures come from `String`-returning builders that call
     `kali_common::array_from_alias_inventory_source` /
     `::array_from_loop_lines`. The builders are private, so the dump lives in a
     module that `include!`s the shipped file -- batch 2's mechanism, and
     `include!` rather than a copy so the executed code is literally the code
     under migration.

  Usage:  t19b5_capture_run.py <out-dir>
          then t19b5_write_captures.py <out-dir>   # writes t19b5_captures.py
"""

from __future__ import annotations

import os
import re
import shutil
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
TESTS = os.path.join(REPO, "crates/kali_cli/tests")

# The `runtime_forin` tests whose fixture is built by a `format!`: the callers of
# the two wrapper helpers. Derived from the source rather than listed, so a new
# caller cannot be missed.
FORMAT_HELPERS = ("forin_ternary_case", "forin_leak_case")


def forin_format_tests() -> list[str]:
    text = open(os.path.join(TESTS, "runtime_forin.rs"), encoding="utf-8").read()
    out = []
    for m in re.finditer(r"#\[test\]\s*\nfn\s+(\w+)\s*\(\s*\)\s*\{", text):
        end = text.find("\n}\n", m.end())
        body = text[m.end():end if end > 0 else len(text)]
        if any(h + "(" in body for h in FORMAT_HELPERS):
            out.append(m.group(1))
    return out


def capture_forin(outdir: str) -> None:
    tests = forin_format_tests()
    print(f"runtime_forin: {len(tests)} format!-built fixture(s)")
    for name in tests:
        tmp = tempfile.mkdtemp(prefix="t19b5-forin-")
        env = dict(os.environ, TMPDIR=tmp)
        subprocess.run(
            ["cargo", "test", "-p", "kali_cli", "--test", "runtime_forin",
             "--", name, "--exact", "--test-threads=1"],
            cwd=REPO, env=env, capture_output=True, text=True)
        found = []
        for d in sorted(os.listdir(tmp)):
            p = os.path.join(tmp, d, "main.ts")
            if os.path.isfile(p):
                found.append(p)
        if len(found) != 1:
            raise SystemExit(
                f"{name}: expected exactly one written fixture, found "
                f"{len(found)} -- the capture is ambiguous and must not be guessed")
        with open(found[0], encoding="utf-8") as f:
            body = f.read()
        with open(os.path.join(outdir, f"forin__{name}.txt"), "w",
                  encoding="utf-8") as f:
            f.write(body)
        shutil.rmtree(tmp, ignore_errors=True)
        print(f"  {name}: {len(body)} byte(s)")


# The dump fn lives INSIDE the module that `include!`s the source: the builders
# are private to that file, so a sibling module cannot name them (batch 2 hit
# the same E0603 and recorded the same resolution).
DUMP = r'''
mod under_test {
    include!("%(src)s");

    pub fn dump() {
        let out = std::env::var("ZZ_OUT").expect("ZZ_OUT");
        let write = |name: &str, body: &str| {
            std::fs::write(std::path::Path::new(&out).join(name), body)
                .expect("write");
        };
        write("spread__array_from_iteration_body.txt",
              &array_from_iteration_body());
        write("spread__browser_harness_array_from_source__run.txt",
              &browser_harness_array_from_source("run"));
        write("spread__browser_harness_array_from_source__test.txt",
              &browser_harness_array_from_source("test"));
        write("spread__set_map_break_continue__run.txt",
              &browser_harness_array_from_set_map_break_continue_source("run"));
        write("spread__set_map_break_continue__test.txt",
              &browser_harness_array_from_set_map_break_continue_source("test"));
    }
}

#[test]
fn zz_dump() {
    under_test::dump();
}
'''


def capture_spread(outdir: str) -> None:
    """`include!` the source through `#[path]` and call its builders.

    The builders are private to the file, so the dump must live in a module that
    carries them; `#[path]` is `include!` with a module boundary, which is what
    lets `under_test::` name them. The temporary target is removed in the same
    run.
    """
    src = os.path.join(TESTS, "for_of_array_iteration_spread.rs")
    dump = os.path.join(TESTS, "zz_t19b5_dump.rs")
    with open(dump, "w", encoding="utf-8") as f:
        f.write(DUMP % {"src": src})
    try:
        r = subprocess.run(
            ["cargo", "test", "-p", "kali_cli", "--test", "zz_t19b5_dump",
             "--", "zz_dump", "--nocapture", "--test-threads=1"],
            cwd=REPO, env=dict(os.environ, ZZ_OUT=outdir),
            capture_output=True, text=True)
        if r.returncode != 0:
            sys.stderr.write(r.stdout + r.stderr)
            raise SystemExit("the dump target failed to build or run")
    finally:
        os.remove(dump)
    n = len([f for f in os.listdir(outdir) if f.startswith("spread__")])
    print(f"for_of_array_iteration_spread: {n} builder output(s)")


def main(argv):
    if not argv:
        raise SystemExit(__doc__)
    outdir = os.path.abspath(argv[0])
    os.makedirs(outdir, exist_ok=True)
    capture_spread(outdir)
    capture_forin(outdir)
    print("CAPTURES TAKEN")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
