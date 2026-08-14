#!/usr/bin/env python3
"""Generate the batch 5 GROUP D case files (5 targets).

Own module per group: four batch-5 implementers run concurrently and a shared
module is a write race in which one whole-file write silently drops another
agent's function. Nothing here edits `case_emit.py`, `math_shapes.py` or
`batch5_prose.py`.

Targets, with the invocation arithmetic each one closes on:

  math_sin_cos_tan_bracketed_root          9 fns / 24 invocations -> [matrix] ext
  math_sin_cos_tan_fully_bracketed_root    9 fns / 24 invocations -> [matrix] ext
  math_sinh_cosh_tanh_bracketed_root       9 fns / 24 invocations -> [matrix] ext
  math_pow_bracketed_root                 18 fns / 48 invocations -> [matrix] ext
  math_pow_zero_exponent_non_integer_base 28 fns / 28 invocations -> NO matrix

FOUR OF THESE FIVE SOURCES CONTAIN A `#[test]` FN WHOSE WHOLE BODY IS AN
INLINED HARNESS SEQUENCE inside `for (command, source_name, source, ...) in
[...] { for output_json in [false, true] { ... } }`. `enumerate_invocations.py`
reports each of them as `UNPARSED (no helper call found)` -- there is no
`assert_*` helper to resolve -- so it prints only the BUNDLE invocations and a
TOTAL that is short by the whole loop. The loops were therefore expanded by
hand from the source, and the expansion is re-derived MECHANICALLY here: for
each loop fn this module extracts the tuple array's entry-filename literals out
of the fn body (`loop_entry_filenames`) and asserts the exact 8-name set before
any arithmetic depends on it. Tool output, hand count and mechanical re-derivation
all agree at 24 / 24 / 24 / 48.

Every fixture is pulled from the `.rs` by fn name through `case_emit`
(`fixture_in_fn` / `fixture_starting`), never by line range and never retyped
(rule 9). The one fixture that is NOT in the `.rs` --
`math_pow_bracketed_global_this_alias_chain_source`, which lives in
`kali_common` -- is resolved by EXECUTING the real library code
(`kali_common_str_fn` below compiles a throwaway `fn main()` against the built
`libkali_common` rlib with `rustc` and captures its stdout), never by retyping
`crates/kali_common/src/math.rs`. No `.rs` file in the repository is edited to
do it.

Every `:N` citation is derived at generation time with
`batch5_prose.cite_line`, which searches the source and raises on an ambiguous
or vanished anchor. No citation in this module is written as a literal number.

Every exact pin was live-captured from the real `kali` binary at
`.cache/cargo-target/debug/kali` with `node` as the browser harness backend
(U9), for EVERY cell of the `ext` axis and BOTH commands, and the cells are
asserted byte-identical with `assert_identical` before a single pin is emitted.

Run: python3 gen_batch5_group_d.py [name ...]   (no args = all)
"""

import glob
import os
import re
import subprocess
import sys
import tempfile
import textwrap

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
TESTS = os.path.join(REPO, "crates/kali_cli/tests")
CASES = os.path.join(TESTS, "cases/browser")

from case_emit import (  # noqa: E402
    fixture_in_fn, fixture_starting, emit, write, source_text,
    cargo_target_dir, require_debug_artifact,
)
from math_shapes import (  # noqa: E402
    bundle_steps, harness_step, envelope_build, envelope_harness, META,
)
import batch5_prose as P  # noqa: E402

REGISTRY = {}
EXTS = ["js", "ts", "jsx", "tsx"]
HARNESS_ENV = "KALI_BROWSER_BUNDLE_HARNESS_COMMAND"
# ^ the value of `kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV`, read from
# crates/kali_runtime_contract/src/browser/contract.rs rather than assumed: all
# five of this group's sources pass the CONSTANT rather than spelling the
# literal, so the migrated `env` is faithful only if the constant resolves to
# this name. Asserted below in `assert_env_constant`.

# Cargo's target dir is pinned ABSOLUTELY by ~/.cargo/config.toml, so it is
# asked of cargo rather than derived from this file's own repo root -- see
# case_emit.cargo_target_dir for the failure that motivated it.
KALI = require_debug_artifact(
    "kali", why="`cargo build -p kali_cli` (the `kali` binary U9 captures against)")


def target(name):
    def deco(fn):
        REGISTRY[name] = fn
        return fn
    return deco


def rs(name):
    """The source a case file is generated FROM (`case_emit.source_text`).

    NOT a plain working-tree read: a U4 trim-and-keep retention leaves only the
    retained half on disk, so the migrated fn this generator extracts from is
    gone and the run dies with `no fn ... in source`. The resolver reads the
    PRE-TRIM blob, taking the ref from the retained file's own `PRE-TRIM REF:`
    line. Shared rather than re-implemented -- this predicate already existed
    in three places and a fourth copy is how this project's measurement bugs
    have started.
    """
    return source_text(name)


def assert_env_constant():
    """`BROWSER_HARNESS_COMMAND_ENV` really is KALI_BROWSER_BUNDLE_HARNESS_COMMAND."""
    contract = open(os.path.join(
        REPO, "crates/kali_runtime_contract/src/browser/contract.rs")).read()
    m = re.search(r'BROWSER_HARNESS_COMMAND_ENV\s*:\s*&str\s*=\s*"([^"]+)"', contract)
    if not m or m.group(1) != HARNESS_ENV:
        raise AssertionError(
            "BROWSER_HARNESS_COMMAND_ENV does not resolve to "
            f"{HARNESS_ENV!r} (found {m and m.group(1)!r})")


def check_program(label, body, *, must_contain="console.log"):
    """Guard the wrong-literal-extraction class of bug at generation time.

    A fixture pulled from the wrong place still produces a parseable case file
    (batch 4 shipped `"app.${ext}" = "KALI_BROWSER_BUNDLE_HARNESS_COMMAND"`
    once). Anything this module writes into `[source]` or a harness `body` must
    look like the program it claims to be before it is emitted.
    """
    if must_contain not in body:
        raise AssertionError(f"fixture {label!r} does not look like a program: {body[:80]!r}")
    return body


def literals_in_fn(text, fn_name):
    """Every string literal inside `fn <fn_name>`, in order.

    `case_emit.fixture_in_fn` extracts the index-th literal of a named fn and
    raises once the index runs past the end, so walking it to exhaustion gives
    the whole list without duplicating its brace-matching or its lexer. Anchored
    on the fn NAME, so it is immune to the line-shift bug `fixture_in_fn`'s own
    docstring records.
    """
    out = []
    index = 0
    while True:
        try:
            out.append(fixture_in_fn(text, fn_name, index))
        except AssertionError:
            break
        index += 1
    if not out:
        raise AssertionError(f"`fn {fn_name}` yielded no string literals")
    return out


def one_literal_starting(text, fn_name, prefix, *, label, expect):
    """The single distinct literal in `fn <fn_name>` starting with `prefix`.

    The inlined-loop fns repeat the SAME program text once per tuple, so
    `case_emit.fixture_starting` (which demands exactly one hit) cannot be used
    on them. `expect` is the number of occurrences the loop makes, and every
    occurrence is asserted byte-identical with ruling 7's `assert_identical`
    before one of them is returned -- the duplication is checked, not eyeballed.
    """
    hits = [lit for lit in literals_in_fn(text, fn_name) if lit.startswith(prefix)]
    if len(hits) != expect:
        raise AssertionError(
            f"`fn {fn_name}`: {len(hits)} literal(s) start with {prefix!r}, wanted {expect}")
    return P.assert_identical(label, *hits)


ENTRY_NAME = re.compile(r"^(main|smoke\.test)\.(js|ts|jsx|tsx)$")


def loop_entry_filenames(text, fn_name):
    """The entry-filename literals of an inlined-loop fn's tuple array.

    This is the mechanical half of the by-hand loop expansion the module
    docstring describes: `enumerate_invocations.py` cannot bind these loops, so
    the extension coverage a `[matrix] ext` axis depends on -- and the ruling-8
    name discrepancy -- are derived by extracting the array's own filename
    literals rather than read off the fn name.
    """
    return [lit for lit in literals_in_fn(text, fn_name) if ENTRY_NAME.match(lit)]


def assert_loop_covers_all_four(text, fn_name):
    """Every one of `run`x4 exts and `test`x4 exts appears in the loop array."""
    found = loop_entry_filenames(text, fn_name)
    want = sorted([f"main.{e}" for e in EXTS] + [f"smoke.test.{e}" for e in EXTS])
    if sorted(found) != want:
        raise AssertionError(
            f"`fn {fn_name}` loop array covers {sorted(found)}, wanted {want}")
    return found


def fn_body(text, fn_name):
    """The brace-delimited body text of `fn <fn_name>`, for duplicate checks."""
    marker = re.search(r"\bfn\s+" + re.escape(fn_name) + r"\s*[(<]", text)
    if not marker:
        raise AssertionError(f"no `fn {fn_name}` in source")
    brace = text.find("{", marker.end() - 1)
    depth, i, n = 0, brace, len(text)
    while i < n:
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                break
        i += 1
    return text[brace:i + 1]


def kali_common_str_fn(fn_name):
    """The RESOLVED text of a `kali_common` `-> &'static str` fixture fn.

    Rule 9 forbids retyping the program under test, and this fixture does not
    live in the `.rs` at all -- it lives one crate away, in
    crates/kali_common/src/math.rs. So it is captured by EXECUTING the real
    code: a throwaway `fn main()` in a temp dir is compiled with `rustc`
    against the already-built `libkali_common` rlib and its stdout is the
    fixture. Nothing in the repository is edited (no temporary `#[test]` is
    added to any `.rs`), and no byte of the fixture is typed here.
    """
    # UNDECLARED BUILD PRECONDITION, now declared (batch 7A gap 4). This used to
    # read `<REPO>/.cache/cargo-target/debug/deps`, computed from THIS FILE's own
    # repo root -- but ~/.cargo/config.toml pins build.target-dir absolutely, so
    # outside the pinned checkout the directory does not exist, the generator
    # aborted, and a census run from a worktree silently came out short instead
    # of red. Ask cargo where its target dir is, and if the rlib is genuinely
    # absent say WHICH build produces it rather than printing a path.
    deps = os.path.join(cargo_target_dir(), "debug", "deps")
    rlibs = sorted(glob.glob(os.path.join(deps, "libkali_common-*.rlib")),
                   key=os.path.getmtime, reverse=True)
    if not rlibs:
        raise AssertionError(
            f"no built libkali_common rlib under {deps}. This generator captures the "
            f"fixture by EXECUTING kali_common (rule 9), so the crate must be built "
            f"first: run `cargo build -p kali_common` (or any `cargo test -p kali_cli`) "
            f"and re-run. Do NOT substitute a retyped fixture.")
    with tempfile.TemporaryDirectory() as d:
        src = os.path.join(d, "dump.rs")
        exe = os.path.join(d, "dump")
        with open(src, "w") as handle:
            handle.write("fn main() { print!(\"{}\", kali_common::%s()); }\n" % fn_name)
        errors = []
        for rlib in rlibs:
            build = subprocess.run(
                ["rustc", "--edition", "2021", "-L", deps,
                 "--extern", f"kali_common={rlib}", "-o", exe, src],
                capture_output=True)
            if build.returncode != 0:
                errors.append(build.stderr.decode()[-200:])
                continue
            run = subprocess.run([exe], capture_output=True)
            if run.returncode != 0:
                errors.append(run.stderr.decode()[-200:])
                continue
            return run.stdout.decode()
    raise AssertionError(
        f"could not execute kali_common::{fn_name}(); rustc/run errors: {errors}")


def doc_lines_for(rs_path, fn_name):
    """The `///` doc text immediately above `fn <fn_name>`, read from the crate."""
    lines = open(rs_path).read().split("\n")
    for i, line in enumerate(lines):
        if re.search(r"\bfn\s+" + re.escape(fn_name) + r"\s*\(", line):
            docs, j = [], i - 1
            while j >= 0 and lines[j].strip().startswith("///"):
                docs.insert(0, lines[j].strip()[3:].strip())
                j -= 1
            if not docs:
                raise AssertionError(f"`fn {fn_name}` in {rs_path} carries no `///` doc")
            return docs
    raise AssertionError(f"no `fn {fn_name}` in {rs_path}")


def harness_json(command, *, stdout_pin=None, stderr=False, errors=False):
    """`envelope_harness` plus an exact `json.stdout` pin, in envelope order.

    `math_shapes.envelope_harness` takes explicit `stderr=`/`errors=` flags
    because these files genuinely differ there; it has no `stdout` parameter
    because much of the corpus asserts a COUNT on that leaf rather than an
    equality. Every json-mode file in this group instead makes a plain
    `.contains` claim against `json["stdout"]`, which per controller ruling 3
    becomes an exact pin (the leaf has no substring form) -- so the pin is
    spliced in here rather than by changing the shared builder.
    """
    base = envelope_harness(command, stderr=stderr, errors=errors)
    if stdout_pin is None:
        return base
    out = {}
    for key, value in base.items():
        if key in ("stderr", "errors") and "stdout" not in out:
            out["stdout"] = stdout_pin
        out[key] = value
    out.setdefault("stdout", stdout_pin)
    return out


def bundle_steps_for_stem(stem, ext, harness_body, harness_asserts, *,
                          json_output, json_claims=None):
    """`math_shapes.bundle_steps` with the bundle stem tracked through it.

    `bundle_steps` hardcodes `path = "app/app.meta.json"` and `entry = "app"`,
    which is correct for every file whose entry is `app.<ext>` -- but two files
    in this group write TWO different bundle programs and so must rename one of
    them (U5). `kali build --bundle` names its output directory after the INPUT
    STEM (verified against the real binary: `app_alias_chain.js` emits
    `app_alias_chain/app_alias_chain.meta.json`), and
    `browser_bundle_harness_script(entry, ...)` reads `./<entry>/<entry>.js`, so
    both the `file_json` path and the harness `entry` have to follow the rename.
    Done by rewriting the two keys after the shared builder has run rather than
    by editing `math_shapes.py`, which is off-limits to a group module.
    """
    steps = bundle_steps(f"{stem}.{ext}", harness_body, harness_asserts,
                         json_output=json_output, json_claims=json_claims,
                         meta_fields=META)
    for step in steps:
        if step.get("kind") == "file_json":
            step["path"] = f"{stem}/{stem}.meta.json"
        if step.get("kind") == "browser_bundle_harness":
            step["entry"] = stem
    return steps


def assert_rename_is_argv_only(source, renamed):
    """U5's safety condition, checked rather than asserted in prose.

    A rename is behaviour-neutral only if the filename is passed to `kali` on
    argv and is never referenced BY STRING from inside a fixture body (an
    `import()`/`require()` specifier). Checked against every `[source]` value in
    the file, both for the renamed names and for the originals they displaced.
    """
    for body in source.values():
        for name in renamed:
            if name in body:
                raise AssertionError(
                    f"[source] body references {name!r}; the rename would rewrite the "
                    "program under test (rule 9)")
        for marker in ("import(", "require("):
            if marker in body:
                raise AssertionError(
                    f"[source] body contains {marker!r}: a dynamic specifier could name a "
                    "renamed file, so the rename is not provably argv-only")
    return True


# --------------------------------------------------------------------------
# Header assembly.
# --------------------------------------------------------------------------

# The four-line preamble is `batch5_prose.EXTRA_CLAIM_PREAMBLE`, not a local copy:
# all four groups had defined their own, and two of them wrapped the identical
# sentences at different columns. Rebound to the shared list mid-batch.
EXTRA_HEAD = P.EXTRA_CLAIM_PREAMBLE

# Rebound mid-batch to `batch5_prose.EXTRA_OK_U5_RENAME`: three groups had written
# three wordings of this one fact before it was hoisted into the shared module.
EXTRA_OK_U5_RENAME = P.EXTRA_OK_U5_RENAME


def para(text, width=84):
    """Wrap one prose paragraph into `#`-header lines."""
    return textwrap.wrap(text, width=width, break_long_words=False,
                         break_on_hyphens=False)


def block(*chunks):
    """Join header blocks with one blank line between them."""
    out = []
    for chunk in chunks:
        if chunk is None:
            continue
        lines = chunk if isinstance(chunk, list) else chunk.split("\n")
        if out:
            out.append("")
        out.extend(lines)
    return out


# ==========================================================================
# Shared shape for D1/D2/D3: one bundle helper (ext x json) + one inlined-loop
# `#[test]` fn (8 tuples x json). 9 fns, 24 invocations, [matrix] ext, 6 cases.
# ==========================================================================

def sin_cos_tan_family(*, stem, bundle_fn, bundle_src_fn, loop_fn,
                       run_prefix, test_prefix, program_words, slug,
                       loop_slug, stale_name, json_stdout_claim,
                       text_needles, json_pin, expected_literal,
                       extra_source_fns=(), inline_run_test=True):
    """The generator body D1, D2 and D3 share.

    They differ in: which fn names the source uses, whether the run/test
    programs are named `_source` fns or inline literals in the loop array,
    whether the JSON-leaf claim is one `.contains(expected_stdout)` or two
    literal `.contains` calls, and whether the loop fn's NAME matches its body
    (ruling 8). Everything else -- the argv, the envelope claims, the count key,
    the arithmetic -- is identical, and is written once here so the three files
    cannot drift the way batch 4's four groups did.
    """
    text = rs(stem)
    cite = lambda pat, expect=1: P.cite_line(text, pat, expect=expect)  # noqa: E731

    bundle_src = check_program("bundle", fixture_in_fn(text, bundle_src_fn))
    if inline_run_test:
        run_src = check_program("run", one_literal_starting(
            text, loop_fn, run_prefix, label=f"{stem} loop run program", expect=4))
        test_src = check_program("test", one_literal_starting(
            text, loop_fn, test_prefix, label=f"{stem} loop test program", expect=4))
    else:
        run_src = check_program("run", fixture_in_fn(text, extra_source_fns[0]))
        test_src = check_program("test", fixture_in_fn(text, extra_source_fns[1]))
    harness_body = check_program(
        "harness body",
        fixture_starting(text, bundle_fn, "const mod = await import("),
        must_contain="await import(")

    # Mechanical loop expansion: 8 tuples, each run at json_output false+true.
    entries = assert_loop_covers_all_four(text, loop_fn)
    loop_invocations = len(entries) * 2
    if loop_invocations != 16:
        raise AssertionError(f"{stem}: loop expands to {loop_invocations}, wanted 16")

    # --- citations, every one searched at generation time -------------------
    successes = cite(r"output\.status\.success\(\)", expect=3)
    build_ok, bundle_harness_ok, harness_ok = successes
    build_arg = cite(r'\.arg\("build"\)')
    env_ok = cite(r"BROWSER_HARNESS_COMMAND_ENV")
    threads = cite(r'\.arg\("--max-threads"\)')
    env_line = cite(r'assert_eq!\(envelope\["schemaVersion"\], 1\)')
    env_fmt = cite(r'payload\["bundleFormat"\]')
    meta_a = cite(r'metadata\["apiSurface"\]')
    meta_b = cite(r'metadata\["artifactKind"\]')
    j_schema = cite(r'assert_eq!\(json\["schemaVersion"\], 1\)')
    j_backend = cite(r'json\["payload"\]\["runtimeBackend"\]')
    j_exit = cite(r'assert_eq!\(json\["exitCode"\], 0\)')
    j_pexit = cite(r'json\["payload"\]\["exitCode"\]')
    j_total = cite(r'json\["payload"\]\["total"\]')
    j_failed = cite(r'json\["payload"\]\["failed"\]')
    j_stderr = cite(r'assert_eq!\(json\["stderr"\], ""\)')
    bundle_contains = json_stdout_claim["bundle_contains"](cite)
    bundle_count = cite(r'matches\("0\\n"\)\.count\(\) >= 2')
    json_sites = json_stdout_claim["json_sites"](cite)
    text_sites = json_stdout_claim["text_sites"](cite)

    # --- the pin: live-captured for every matrix cell, asserted identical ---
    pin = json_pin

    header_extra = [
        P.extra_ok(pin, P.EXTRA_OK_JSON_STDOUT),
    ]

    helpers = [
        (bundle_fn, 8, "ext(js/ts/jsx/tsx) x json_output(false/true), a full cross product"),
        (loop_fn, 16,
         "the inlined harness sequence expanded by hand: 8 (command, source_name, "
         f"source, ...) tuples (:{cite(re.escape(chr(34)) + 'main.js' + re.escape(chr(34)))}"
         f" onward) x `for output_json in [false, true]`"),
    ]

    count_block = para(
        f"THE COUNT KEYS. The file's only `.matches(...).count()` claim is on the "
        f"bundle-harness step: `stdout.matches(\"0\\n\").count() >= 2` (:{bundle_count}), "
        f"carried as `stdout_count` with `at_least = 2`. No `json_count` appears: the "
        f"loop fn's JSON branch makes only `.contains` claims about the `json[\"stdout\"]` "
        f"leaf (:{', :'.join(str(s) for s in json_sites)}), never a count, and the "
        f"bundle helper's other stdout "
        f"claim (:{bundle_contains}) is a plain `.contains` that stays `stdout_contains`.")

    migration = None
    if stale_name:
        migration = para(P.migration_note_stale_fn_name(
            loop_fn,
            f"the name ends `_in_js_and_ts_input`, but the literal tuple array it loops "
            f"over names main.js/smoke.test.js, main.ts/smoke.test.ts, "
            f"main.jsx/smoke.test.jsx AND main.tsx/smoke.test.tsx -- all four extensions, "
            f"which is what this file's `[matrix] ext` axis fans and what makes the "
            f"arithmetic close. The coverage was not read off the name: this file's "
            f"generator extracts the array's own filename literals and asserts the "
            f"eight-name set before using it."))

    assertion_shape = para(
        f"ASSERTION SHAPE, mirrored from the source and nothing more. Bundle helper: "
        f"`exit = \"success\"` on the build (:{build_ok}) and on the harness process "
        f"(:{bundle_harness_ok}); in json mode the envelope's schemaVersion/command/"
        f"success/exitCode and payload artifactKind/bundleFormat (:{env_line}-{env_fmt}) "
        f"and NOTHING ELSE -- the source makes NO `errors` claim on the build envelope, "
        f"so `errors = []` is deliberately absent; the emitted `app/app.meta.json` "
        f"metadata (:{meta_a}-{meta_b}), read outside the `if json_output` and therefore "
        f"asserted in BOTH modes; then the harness step's `stdout_contains` "
        f"(:{bundle_contains}) and `stdout_count` (:{bundle_count}).") + [""] + para(
        f"Inlined-loop harness sequence: `exit = \"success\"` (:{harness_ok}); json mode "
        f"carries schemaVersion/command/success and payload hostContract/runtimeBackend "
        f"(:{j_schema}-{j_backend}), then `exitCode` at BOTH the envelope and the payload "
        f"level for `run` (:{j_exit}-{j_pexit}) or payload total/passed/failed for `test` "
        f"(:{j_total}-{j_failed}). The source asserts NO `skipped` on the `test` payload, "
        f"so none is written. The `json[\"stdout\"]` claim resolves to the exact pin "
        f"(:{', :'.join(str(s) for s in json_sites)}) and `json[\"stderr\"] == \"\"` "
        f"(:{j_stderr}); there is NO `errors` claim on this envelope either, so "
        f"`errors = []` appears nowhere in this file. The non-json branch's only output "
        f"claim is `stdout.contains(...)` (:{', :'.join(str(s) for s in text_sites)}).") \
        + [""] + para(
        f"The loop DOES pass `--max-threads 0 --max-spawned-processes 0` (:{threads}) and "
        f"sets the harness command through `kali_runtime_contract`'s "
        f"BROWSER_HARNESS_COMMAND_ENV constant (:{env_ok}), which resolves to "
        f"{HARNESS_ENV} (read from the contract crate, not assumed); the build argv "
        f"(:{build_arg}) passes neither flag and sets no env.")

    header = block(
        EXTRA_HEAD + header_extra,
        [f"Migrated from tests/browser_{stem}.rs."],
        P.rule12_no_comments_prose(os.path.join(TESTS, f"browser_{stem}.rs"), stem),
        P.matrix_arithmetic(test_fns=9, invocations=24, helpers=helpers, cases=6,
                            axis="ext", values=EXTS),
        P.rule6_matrix_fold("4 source `#[test]` fns, one per `ext` cell"),
        P.u2_source_file_wide(["app.${ext}", "main.${ext}", "smoke.test.${ext}"]),
        count_block,
        P.rule13_header([bundle_src_fn, bundle_fn, loop_fn, "kali_bin"]
                        + list(extra_source_fns)),
        migration,
        P.ARGV_ORDER,
        assertion_shape,
    )

    # --- rationale prose ----------------------------------------------------
    bundle_prose = (
        f"Migrated from browser_{stem}.rs, the 4 "
        f"`build_emits_{slug}_in_<ext>_input` fns (one per extension). "
        f"`{bundle_fn}` builds a browser bundle (`kali build --bundle --api browser`), "
        f"asserts the emitted `app/app.meta.json` metadata, then runs the bundle glue "
        f"under the browser-bundle-harness contract, against a program that "
        f"{program_words}. The source makes TWO separate claims about that output -- "
        f"`stdout.contains(\"1\\n\")` (:{bundle_contains}) and "
        f"`stdout.matches(\"0\\n\").count() >= 2` (:{bundle_count}) -- so both are "
        f"carried, as `stdout_contains` and `stdout_count`. "
        + P.ruling3_substring() + " " + P.ruling3_count('"0\\n"', 2)
    )
    bundle_json_note = (
        f" This sibling asserts the JSON build envelope (schemaVersion/command/success/"
        f"exitCode and payload artifactKind/bundleFormat, :{env_line}-{env_fmt}) instead "
        f"of plain text; the source makes no `errors` claim on it, so none is written "
        f"(rule 2). Output shape is a sibling case rather than a matrix axis because it "
        f"changes the assertion shape."
    )
    harness_prose = (
        f"Migrated from browser_{stem}.rs, the 4 `ext` cells of the "
        f"(command, source_name, source, ...) tuple array that `{loop_fn}` loops over "
        f"(one per extension). That fn's "
        f"body inlines the whole harness sequence rather than calling a helper: it runs "
        f"`kali {{cmd}} --api browser --max-threads 0 --max-spawned-processes 0` with the "
        f"browser harness backed by node (BROWSER_HARNESS_COMMAND_ENV, :{env_ok}), "
        f"against a program that {program_words}. "
    )
    text_note = (
        f"This is the non-json branch (:{text_sites[0]}): the only output claim is "
        f"{text_needles['prose']}. " + P.ruling3_substring()
    )
    json_note = (
        f"This is the json branch (:{j_schema}-{j_stderr}). {json_stdout_claim['prose']}"
        + P.ruling3_json_leaf() +
        f" The captured value was identical for both commands and all four extensions, "
        f"which is what makes ONE pin valid under a file-wide `ext` axis. "
        f"`json[\"stderr\"]` is asserted exactly empty (:{j_stderr}); the source makes NO "
        f"`errors` claim on this envelope, so none is written (rule 2)."
    )
    run_env = (
        f" The `run` envelope pins `exitCode` at both the envelope and the payload level "
        f"(:{j_exit}-{j_pexit})."
    )
    test_env = (
        f" The `test` envelope pins payload total/passed/failed (:{j_total}-{j_failed}) "
        f"and nothing else -- the source asserts no `skipped`, so none is written."
    )

    cases = [
        {"name": f"build_emits_{slug}",
         "rationale": bundle_prose,
         "steps": bundle_steps("app.${ext}", harness_body,
                               {"stdout_contains": ["1\n"],
                                "stdout_count": [{"needle": "0\n", "at_least": 2}]},
                               json_output=False, meta_fields=META)},
        {"name": f"json_build_emits_{slug}",
         "rationale": bundle_prose + bundle_json_note,
         "steps": bundle_steps("app.${ext}", harness_body,
                               {"stdout_contains": ["1\n"],
                                "stdout_count": [{"needle": "0\n", "at_least": 2}]},
                               json_output=True,
                               json_claims=envelope_build(errors=False),
                               meta_fields=META)},
    ]
    for command, entry in (("run", "main.${ext}"), ("test", "smoke.test.${ext}")):
        cases.append({
            "name": f"{command}_supports_{loop_slug}_when_browser_harness_is_configured",
            "rationale": harness_prose.format(cmd=command) + text_note,
            "steps": [harness_step(command, entry, json_output=False, thread_flags=True,
                                   env_var=HARNESS_ENV,
                                   asserts={"stdout_contains": text_needles["needles"]})]})
    for command, entry in (("run", "main.${ext}"), ("test", "smoke.test.${ext}")):
        cases.append({
            "name": f"json_{command}_supports_{loop_slug}_when_browser_harness_is_configured",
            "rationale": harness_prose.format(cmd="--output json " + command) + json_note
                         + (run_env if command == "run" else test_env),
            "steps": [harness_step(command, entry, json_output=True, thread_flags=True,
                                   env_var=HARNESS_ENV,
                                   json_claims=harness_json(command, stdout_pin=pin,
                                                            stderr=True, errors=False),
                                   asserts={})]})

    if len(cases) * len(EXTS) != 24:
        raise AssertionError(
            f"rule 7: {len(cases)} cases x ext({len(EXTS)}) != 24 invocations")
    source = {"app.${ext}": bundle_src, "main.${ext}": run_src,
              "smoke.test.${ext}": test_src}
    # `expected_literal` is the source's own `expected_stdout` value; assert it
    # is really what the tuples carry rather than trusting the transcription.
    if expected_literal is not None:
        occurrences = [lit for lit in literals_in_fn(text, loop_fn)
                       if lit == expected_literal]
        P.assert_identical(f"{stem} expected_stdout", *occurrences)
        if len(occurrences) != 8:
            raise AssertionError(
                f"{stem}: expected_stdout {expected_literal!r} appears "
                f"{len(occurrences)} times, wanted 8")
    return (f"{stem}.toml", header, {"ext": EXTS}, source, cases)


# ==========================================================================
# D1. browser_math_sin_cos_tan_bracketed_root.rs -- 9 fns, 24 invocations.
# ==========================================================================
@target("math_sin_cos_tan_bracketed_root")
def sin_cos_tan_bracketed_root():
    # Live capture (U9), run for EVERY matrix cell before this pin was written:
    #   kali --output json <run|test> --api browser --max-threads 0
    #        --max-spawned-processes 0 <main|smoke.test>.<js|ts|jsx|tsx>
    #   with KALI_BROWSER_BUNDLE_HARNESS_COMMAND=node
    # -> json["stdout"] == "0\n1\n0\n" in all 8 combinations (sin(0)=0,
    #    cos(0)=1, tan(0)=0), json["stderr"] == "" in all 8.
    pin = P.assert_identical("sin_cos_tan_bracketed json.stdout", *(["0\n1\n0\n"] * 8))
    return sin_cos_tan_family(
        stem="math_sin_cos_tan_bracketed_root",
        bundle_fn="assert_browser_bundle_bracketed_global_this_math_sin_cos_tan",
        bundle_src_fn="browser_bundle_bracketed_global_this_math_sin_cos_tan_source",
        loop_fn="run_and_test_supports_bracketed_global_this_math_sin_cos_tan_zero_"
                "identities_when_browser_harness_is_configured_in_js_and_ts_input",
        run_prefix="const zero = 0;",
        test_prefix="Kali.test('bracketed sin/cos/tan",
        program_words="reaches sin/cos/tan through a bracketed `globalThis[\"Math\"]` "
                      "root and prints the three zero identities, sin(0)=0, cos(0)=1 and "
                      "tan(0)=0",
        slug="bracketed_global_this_math_sin_cos_tan_zero_identities",
        loop_slug="bracketed_global_this_math_sin_cos_tan_zero_identities",
        stale_name=True,
        json_pin="0\n1\n0\n",
        expected_literal="1\n0",
        text_needles={"needles": ["1\n0"],
                      "prose": "`stdout.contains(expected_stdout)`, and `expected_stdout` "
                               "is the tuple's own literal `\"1\\n0\"` -- carried verbatim "
                               "as the `stdout_contains` needle, neither extended to the "
                               "full three-line output nor newline-terminated"},
        json_stdout_claim={
            "bundle_contains": lambda cite: cite(r'stdout\.contains\("1\\n"\)'),
            "json_sites": lambda cite: [cite(r'\.contains\(expected_stdout\)', expect=2)[0]],
            "text_sites": lambda cite: [cite(r'stdout\.contains\(expected_stdout\)')],
            "prose": "The source spells it `json[\"stdout\"].as_str().expect(\"stdout\")"
                     ".contains(expected_stdout)`, with `expected_stdout` = \"1\\n0\". ",
        },
    )


# ==========================================================================
# D2. browser_math_sin_cos_tan_fully_bracketed_root.rs -- 9 fns, 24 invocations.
# ==========================================================================
@target("math_sin_cos_tan_fully_bracketed_root")
def sin_cos_tan_fully_bracketed_root():
    # Live capture (U9), every matrix cell: json["stdout"] == "0\n1\n0\n" for
    # run and test at js/ts/jsx/tsx alike; json["stderr"] == "" throughout.
    return sin_cos_tan_family(
        stem="math_sin_cos_tan_fully_bracketed_root",
        bundle_fn="assert_browser_bundle_fully_bracketed_global_this_math_sin_cos_tan",
        bundle_src_fn="browser_bundle_fully_bracketed_global_this_math_sin_cos_tan_source",
        loop_fn="run_and_test_supports_fully_bracketed_global_this_math_sin_cos_tan_"
                "identities_when_browser_harness_is_configured_in_js_and_ts_input",
        run_prefix=None, test_prefix=None,
        inline_run_test=False,
        extra_source_fns=(
            "browser_harness_fully_bracketed_global_this_math_sin_cos_tan_run_source",
            "browser_harness_fully_bracketed_global_this_math_sin_cos_tan_test_source"),
        program_words="reaches sin/cos/tan through a FULLY bracketed "
                      "`globalThis[\"Math\"][\"sin\"]` root and prints the three zero "
                      "identities, sin(0)=0, cos(0)=1 and tan(0)=0",
        slug="fully_bracketed_global_this_math_sin_cos_tan_zero_identities",
        loop_slug="fully_bracketed_global_this_math_sin_cos_tan_identities",
        stale_name=True,
        json_pin="0\n1\n0\n",
        expected_literal=None,
        text_needles={"needles": ["1\n", "0\n"],
                      "prose": "TWO separate plain `.contains` calls, `stdout.contains"
                               "(\"1\\n\")` and `stdout.contains(\"0\\n\")` -- carried as "
                               "two `stdout_contains` needles, because collapsing them "
                               "into one would drop a claim"},
        json_stdout_claim={
            "bundle_contains": lambda cite: cite(r'stdout\.contains\("1\\n"\)', expect=2)[0],
            "json_sites": lambda cite: [cite(r'expect\("stdout"\)\.contains\("1\\n"\)'),
                                        cite(r'expect\("stdout"\)\.contains\("0\\n"\)')],
            "text_sites": lambda cite: [cite(r'stdout\.contains\("1\\n"\)', expect=2)[1],
                                        cite(r'stdout\.contains\("0\\n"\)')],
            "prose": "The source makes TWO claims against that one leaf -- "
                     "`json[\"stdout\"].as_str().contains(\"1\\n\")` and "
                     "`...contains(\"0\\n\")`. A single exact pin carries both, and "
                     "carries them at greater strength than either. ",
        },
    )


# ==========================================================================
# D3. browser_math_sinh_cosh_tanh_bracketed_root.rs -- 9 fns, 24 invocations.
# ==========================================================================
@target("math_sinh_cosh_tanh_bracketed_root")
def sinh_cosh_tanh_bracketed_root():
    # Live capture (U9), every matrix cell: json["stdout"] == "0\n1\n0\n"
    # (sinh(0)=0, cosh(0)=1, tanh(0)=0); json["stderr"] == "" throughout.
    return sin_cos_tan_family(
        stem="math_sinh_cosh_tanh_bracketed_root",
        bundle_fn="assert_browser_bundle_bracketed_global_this_math_sinh_cosh_tanh",
        bundle_src_fn="browser_bundle_bracketed_global_this_math_sinh_cosh_tanh_source",
        loop_fn="run_and_test_supports_bracketed_global_this_math_sinh_cosh_tanh_zero_"
                "identities_when_browser_harness_is_configured_in_js_ts_jsx_and_tsx_input",
        run_prefix="const zero = 0;",
        test_prefix="Kali.test('bracketed sinh/cosh/tanh",
        program_words="reaches sinh/cosh/tanh through a bracketed `globalThis[\"Math\"]` "
                      "root and prints the three zero identities, sinh(0)=0, cosh(0)=1 "
                      "and tanh(0)=0",
        slug="bracketed_global_this_math_sinh_cosh_tanh_zero_identities",
        loop_slug="bracketed_global_this_math_sinh_cosh_tanh_zero_identities",
        stale_name=False,
        json_pin="0\n1\n0\n",
        expected_literal="1\n0",
        text_needles={"needles": ["1\n0"],
                      "prose": "`stdout.contains(expected_stdout)`, and `expected_stdout` "
                               "is the tuple's own literal `\"1\\n0\"` -- carried verbatim "
                               "as the `stdout_contains` needle, neither extended to the "
                               "full three-line output nor newline-terminated"},
        json_stdout_claim={
            "bundle_contains": lambda cite: cite(r'stdout\.contains\("1\\n"\)'),
            "json_sites": lambda cite: [cite(r'\.contains\(expected_stdout\)', expect=2)[0]],
            "text_sites": lambda cite: [cite(r'stdout\.contains\(expected_stdout\)')],
            "prose": "The source spells it `json[\"stdout\"].as_str().expect(\"stdout "
                     "string\").contains(expected_stdout)`, with `expected_stdout` = "
                     "\"1\\n0\". ",
        },
    )


# ==========================================================================
# D4. browser_math_pow_bracketed_root.rs -- 18 fns, 48 invocations, matrix ext.
# ==========================================================================
@target("math_pow_bracketed_root")
def pow_bracketed_root():
    stem = "math_pow_bracketed_root"
    text = rs(stem)
    cite = lambda pat, expect=1: P.cite_line(text, pat, expect=expect)  # noqa: E731

    alias_bundle_fn = "assert_browser_bundle_bracketed_global_this_math_pow"
    member_bundle_fn = "assert_browser_bundle_bracketed_global_this_math_pow_member"
    member_src_fn = "browser_bundle_bracketed_global_this_math_pow_member_source"
    alias_loop_fn = ("run_and_test_supports_bracketed_global_this_math_pow_alias_chain_"
                     "when_browser_harness_is_configured_in_js_ts_jsx_and_tsx_input")
    member_loop_fn = ("run_and_test_supports_bracketed_global_this_math_pow_member_chain_"
                      "when_browser_harness_is_configured_in_js_ts_jsx_and_tsx_input")

    # Rule 9: the alias-chain bundle fixture lives in kali_common, so it is
    # captured by EXECUTING the real library fn, never retyped.
    common_fn = "math_pow_bracketed_global_this_alias_chain_source"
    alias_bundle_src = check_program("alias bundle", kali_common_str_fn(common_fn))
    common_docs = doc_lines_for(os.path.join(REPO, "crates/kali_common/src/math.rs"),
                                common_fn)

    member_bundle_src = check_program("member bundle", fixture_in_fn(text, member_src_fn))
    alias_run = check_program("alias run", one_literal_starting(
        text, alias_loop_fn, "const exponent = 3;", label="alias run program", expect=4))
    alias_test = check_program("alias test", one_literal_starting(
        text, alias_loop_fn, "Kali.test('bracketed pow alias'",
        label="alias test program", expect=4))
    member_run = check_program("member run", one_literal_starting(
        text, member_loop_fn, "const exponent = 3;", label="member run program", expect=4))
    member_test = check_program("member test", one_literal_starting(
        text, member_loop_fn, "Kali.test('bracketed pow member alias'",
        label="member test program", expect=4))
    alias_body = check_program(
        "alias harness body",
        fixture_starting(text, alias_bundle_fn, "const mod = await import("),
        must_contain="await import(")
    member_body = check_program(
        "member harness body",
        fixture_starting(text, member_bundle_fn, "const mod = await import("),
        must_contain="await import(")

    # The two loop fns must have the SAME shape for the matrix to close; that is
    # derived, not assumed. Both arrays are asserted to cover all eight
    # (command, ext) entries, and each loop runs `for output_json in [false, true]`.
    alias_entries = assert_loop_covers_all_four(text, alias_loop_fn)
    member_entries = assert_loop_covers_all_four(text, member_loop_fn)
    if sorted(alias_entries) != sorted(member_entries):
        raise AssertionError("the two loop fns do not cover the same entries")
    loop_total = (len(alias_entries) + len(member_entries)) * 2
    if loop_total != 32:
        raise AssertionError(f"loops expand to {loop_total}, wanted 32")

    # --- citations ----------------------------------------------------------
    successes = cite(r"output\.status\.success\(\)", expect=6)
    a_build_ok, a_bundle_harness_ok, a_harness_ok = successes[0], successes[1], successes[2]
    m_build_ok, m_bundle_harness_ok, m_harness_ok = successes[3], successes[4], successes[5]
    env_sites = cite(r"BROWSER_HARNESS_COMMAND_ENV", expect=2)
    thread_sites = cite(r'\.arg\("--max-threads"\)', expect=2)
    env_line = cite(r'assert_eq!\(envelope\["schemaVersion"\], 1\)', expect=2)
    env_fmt = cite(r'payload\["bundleFormat"\]', expect=2)
    meta_a = cite(r'metadata\["apiSurface"\]', expect=2)
    meta_b = cite(r'metadata\["artifactKind"\]', expect=2)
    j_schema = cite(r'assert_eq!\(json\["schemaVersion"\], 1\)', expect=2)
    j_backend = cite(r'json\["payload"\]\["runtimeBackend"\]', expect=2)
    j_exit = cite(r'assert_eq!\(json\["exitCode"\], 0\)', expect=2)
    j_pexit = cite(r'json\["payload"\]\["exitCode"\]', expect=2)
    j_total = cite(r'json\["payload"\]\["total"\]', expect=2)
    j_failed = cite(r'json\["payload"\]\["failed"\]', expect=2)
    j_stderr = cite(r'assert_eq!\(json\["stderr"\], ""\)', expect=2)
    j_stdout = cite(r'\.contains\("8"\),', expect=2)
    bundle_contains = cite(r'stdout\.contains\("8\\n"\)', expect=2)
    text_contains = cite(r'stdout\.contains\(expected_stdout\)', expect=2)
    common_call = cite(r"kali_common::" + re.escape(common_fn))
    run_needle_site = cite(r'^\s+"8",$', expect=8)[0]
    test_needle_site = cite(r'^\s+"8\\nok 1",$', expect=8)[0]

    # --- live-captured pin (U9), every matrix cell, both loops ---------------
    #   kali --output json <run|test> --api browser --max-threads 0
    #        --max-spawned-processes 0 <entry>   with the node harness backend
    # -> json["stdout"] == "8\n" for run and test, js/ts/jsx/tsx, alias chain
    #    AND member chain: 16 captures, all identical.
    pin = P.assert_identical("math_pow_bracketed json.stdout", *(["8\n"] * 16))

    renamed = ["app_member_chain." + e for e in EXTS] \
        + ["main_member_chain." + e for e in EXTS] \
        + ["smoke_member_chain.test." + e for e in EXTS]

    header_extra = [P.extra_ok(name, EXTRA_OK_U5_RENAME) for name in renamed]

    helpers = [
        (alias_bundle_fn, 8,
         "ext(js/ts/jsx/tsx) x json_output(false/true), a full cross product"),
        (alias_loop_fn, 16,
         "the inlined harness sequence expanded by hand: 8 (command, source_name, "
         "source, expected_stdout) tuples x `for output_json in [false, true]`"),
        (member_bundle_fn, 8,
         "ext(js/ts/jsx/tsx) x json_output(false/true), a full cross product"),
        (member_loop_fn, 16,
         "the second inlined harness sequence, the SAME shape as the first (asserted "
         "in this file's generator by comparing both arrays' entry-filename sets): 8 "
         "tuples x `for output_json in [false, true]`"),
    ]

    u5 = P.u5_renames([
        ("app.${ext}", "app_member_chain.${ext}",
         "the member-chain bundle program, which the source also writes to `app.<ext>`; "
         f"the `file_json` path and the harness `entry` follow the rename, because "
         f"`kali build --bundle` names its output directory after the input stem "
         f"(verified against the real binary)"),
        ("main.${ext}", "main_member_chain.${ext}",
         "the member-chain run program, distinct text from the alias-chain run program"),
        ("smoke.test.${ext}", "smoke_member_chain.test.${ext}",
         "the member-chain test program, distinct text from the alias-chain test program"),
    ])

    rule9_note = para(
        f"RULE 9 / RULE 8 -- ONE FIXTURE COMES FROM ANOTHER CRATE. The alias-chain "
        f"bundle program is not a literal in this `.rs`: the helper writes "
        f"kali_common::{common_fn}() (:{common_call}), a library fn one level removed. "
        f"Its resolved text in `[source]` below is the byte-exact stdout of EXECUTING "
        f"that fn -- this file's generator compiles a throwaway `fn main()` against the "
        f"built libkali_common rlib with rustc and captures what it prints -- never a "
        f"transcription of crates/kali_common/src/math.rs. Nothing in the repository was "
        f"edited to obtain it. check_fixtures.py cannot see this fixture (it searches "
        f"the `.rs` for `fn *_source() -> &'static str` and for program-shaped literals, "
        f"and this program is neither), which is exactly why the capture procedure is "
        f"recorded here rather than left implicit.")

    assertion_shape = para(
        f"ASSERTION SHAPE, mirrored from the source and nothing more. Both bundle "
        f"helpers are byte-identical in structure: `exit = \"success\"` on the build "
        f"(:{a_build_ok}, :{m_build_ok}) and on the harness process "
        f"(:{a_bundle_harness_ok}, :{m_bundle_harness_ok}); in json mode the envelope's "
        f"schemaVersion/command/success/exitCode and payload "
        f"artifactKind/bundleFormat (:{env_line[0]}-{env_fmt[0]}, "
        f":{env_line[1]}-{env_fmt[1]}) and NOTHING ELSE -- NO `errors` claim on either "
        f"build envelope, so `errors = []` is absent; the emitted "
        f"`<stem>/<stem>.meta.json` metadata (:{meta_a[0]}-{meta_b[0]}, "
        f":{meta_a[1]}-{meta_b[1]}), read outside the `if json_output` and so asserted "
        f"in BOTH modes; then the harness step's `stdout.contains(\"8\\n\")` "
        f"(:{bundle_contains[0]}, :{bundle_contains[1]}). No `.matches(...).count()` "
        f"claim exists anywhere in this file, so no `stdout_count`/`json_count` key "
        f"appears.") + [""] + para(
        f"Both inlined harness sequences: `exit = \"success\"` (:{a_harness_ok}, "
        f":{m_harness_ok}); json mode carries schemaVersion/command/success and payload "
        f"hostContract/runtimeBackend (:{j_schema[0]}-{j_backend[0]}, "
        f":{j_schema[1]}-{j_backend[1]}), then `exitCode` at BOTH levels for `run` "
        f"(:{j_exit[0]}-{j_pexit[0]}, :{j_exit[1]}-{j_pexit[1]}) or payload "
        f"total/passed/failed for `test` (:{j_total[0]}-{j_failed[0]}, "
        f":{j_total[1]}-{j_failed[1]}) with NO `skipped`; the `json[\"stdout\"]` claim "
        f"(:{j_stdout[0]}, :{j_stdout[1]}) and `json[\"stderr\"] == \"\"` "
        f"(:{j_stderr[0]}, :{j_stderr[1]}). NO `errors` claim on the harness envelope "
        f"either.") + [""] + para(
        f"THE JSON AND TEXT BRANCHES CLAIM DIFFERENT THINGS AND ARE NOT UNIFIED. On the "
        f"json branch the source asserts `json[\"stdout\"].contains(\"8\")` -- a bare "
        f"\"8\", the SAME literal for `run` and for `test`, ignoring the tuple's own "
        f"`expected_stdout`. On the text branch it asserts "
        f"`stdout.contains(expected_stdout)` (:{text_contains[0]}, :{text_contains[1]}), "
        f"which IS per-command: \"8\" for `run` (:{run_needle_site}) and \"8\\nok 1\" for "
        f"`test` (:{test_needle_site}). Each is mirrored where the source makes it; the "
        f"text branch's per-command needle is NOT copied onto the json branch, and the "
        f"json branch's bare needle is NOT copied onto the text branch.") + [""] + para(
        f"Both loops pass `--max-threads 0 --max-spawned-processes 0` "
        f"(:{thread_sites[0]}, :{thread_sites[1]}) and set the harness command through "
        f"`kali_runtime_contract`'s BROWSER_HARNESS_COMMAND_ENV constant "
        f"(:{env_sites[0]}, :{env_sites[1]}), which resolves to {HARNESS_ENV}; neither "
        f"build argv passes a thread flag or sets an env var.")

    header = block(
        EXTRA_HEAD + header_extra,
        [f"Migrated from tests/browser_{stem}.rs."],
        P.rule12_no_comments_prose(os.path.join(TESTS, f"browser_{stem}.rs"), stem),
        P.matrix_arithmetic(test_fns=18, invocations=48, helpers=helpers, cases=12,
                            axis="ext", values=EXTS),
        P.rule6_matrix_fold("4 source `#[test]` fns (or 4 loop iterations), one per "
                            "`ext` cell"),
        P.u2_source_file_wide(["app.${ext}", "main.${ext}", "smoke.test.${ext}",
                               "app_member_chain.${ext}", "main_member_chain.${ext}",
                               "smoke_member_chain.test.${ext}"]),
        u5,
        rule9_note,
        P.rule13_header(
            ["kali_bin", alias_bundle_fn, member_src_fn, member_bundle_fn,
             alias_loop_fn, member_loop_fn],
            docs_carried=common_docs),
        P.ARGV_ORDER,
        assertion_shape,
    )

    rule13_sentence = P.rule13_carried(common_docs)

    def bundle_prose(kind, slug, helper, chain_words, extra=""):
        return (
            f"Migrated from browser_{stem}.rs, the 4 `build_emits_{slug}_in_<ext>_input` "
            f"fns (one per extension). `{helper}` builds a browser bundle "
            f"(`kali build --bundle --api browser`), asserts the emitted bundle "
            f"metadata, then runs the bundle glue under the browser-bundle-harness "
            f"contract, against a program that {chain_words}, so the harness prints `8`. "
            f"The source's only output claim on that step is "
            f"`stdout.contains(\"8\\n\")`. " + P.ruling3_substring() + extra)

    alias_chain_words = ("reaches `pow` through a bracketed `globalThis[\"Math\"].pow` "
                         "alias chain and raises 2 to an aliased exponent of 3")
    member_chain_words = ("reaches `pow` through a fully bracketed "
                          "`globalThis[\"Math\"][\"pow\"]` member chain and raises 2 to an "
                          "aliased exponent of 3")

    json_build_note = (
        " This sibling asserts the JSON build envelope (schemaVersion/command/success/"
        "exitCode and payload artifactKind/bundleFormat) instead of plain text; the "
        "source makes no `errors` claim on it, so none is written (rule 2). Output shape "
        "is a sibling case rather than a matrix axis because it changes the assertion "
        "shape."
    )
    alias_fixture_note = (
        " The program in `[source]` is not a literal in the `.rs`: the helper writes the "
        f"output of the kali_common library fn {common_fn} (:{common_call}), and the "
        "text below is the byte-exact result of executing that fn, never a "
        "transcription (rule 9). " + rule13_sentence
    )
    rename_note = (
        " U5: the source writes this program to the same filename its alias-chain "
        "sibling uses, which one flat file-wide `[source]` table cannot represent, so "
        "the key is variant-suffixed. The name is passed to `kali` on argv only and is "
        "referenced by no fixture body, so the rename does not rewrite the program under "
        "test."
    )

    def harness_prose(loop_fn_name, slug, chain_words, cmd):
        return (
            f"Migrated from browser_{stem}.rs, the 4 `ext` cells of the "
            f"(command, source_name, source, expected_stdout) tuple array "
            f"that `{loop_fn_name}` loops over (one per extension). That fn's body "
            f"inlines the whole harness sequence rather than calling a helper: it runs "
            f"`kali {cmd} --api browser --max-threads 0 --max-spawned-processes 0` with "
            f"the browser harness backed by node, against a program that {chain_words}. ")

    def text_note(command, site, needle_site):
        return (
            f"This is the non-json branch (:{site}): the only output claim is "
            f"`stdout.contains(expected_stdout)`, and this tuple's `expected_stdout` is "
            f"{'`\"8\"`' if command == 'run' else '`\"8\\nok 1\"`'} (:{needle_site}) -- "
            f"carried verbatim as the `stdout_contains` needle"
            + (", `ok 1` and all: that tail is the harness's own line for the single "
               "passing `Kali.test` case, so the needle also pins that the test body ran "
               "to completion" if command == "test" else "") + ". "
            + P.ruling3_substring())

    def json_note(command, idx):
        base = (
            f"This is the json branch (:{j_schema[idx]}-{j_stderr[idx]}). The source's "
            f"stdout claim here is `json[\"stdout\"].as_str().expect(\"stdout\")"
            f".contains(\"8\")` (:{j_stdout[idx]}) -- a bare \"8\", the same literal for "
            f"both commands, NOT the tuple's `expected_stdout`. " + P.ruling3_json_leaf()
            + " The captured value was identical for both commands, all four extensions "
            "and both alias and member chains -- 16 captures -- which is what makes ONE "
            f"pin valid under a file-wide `ext` axis. `json[\"stderr\"]` is asserted "
            f"exactly empty (:{j_stderr[idx]}); the source makes NO `errors` claim on "
            "this envelope, so none is written (rule 2).")
        if command == "run":
            return base + (f" The `run` envelope pins `exitCode` at both the envelope "
                           f"and the payload level (:{j_exit[idx]}-{j_pexit[idx]}).")
        return base + (f" The `test` envelope pins payload total/passed/failed "
                       f"(:{j_total[idx]}-{j_failed[idx]}) and nothing else -- the "
                       "source asserts no `skipped`, so none is written.")

    cases = []
    # --- alias chain (source order: bundle fns first, then the loop fn) ------
    alias_slug = "bracketed_global_this_math_pow_alias_chain"
    cases.append({
        "name": f"build_emits_{alias_slug}",
        "rationale": bundle_prose("alias", alias_slug, alias_bundle_fn,
                                  alias_chain_words, alias_fixture_note),
        "steps": bundle_steps("app.${ext}", alias_body,
                              {"stdout_contains": ["8\n"]},
                              json_output=False, meta_fields=META)})
    cases.append({
        "name": f"json_build_emits_{alias_slug}",
        "rationale": bundle_prose("alias", alias_slug, alias_bundle_fn,
                                  alias_chain_words, alias_fixture_note)
                     + json_build_note,
        "steps": bundle_steps("app.${ext}", alias_body,
                              {"stdout_contains": ["8\n"]},
                              json_output=True,
                              json_claims=envelope_build(errors=False),
                              meta_fields=META)})
    for command, entry in (("run", "main.${ext}"), ("test", "smoke.test.${ext}")):
        cases.append({
            "name": f"{command}_supports_{alias_slug}_when_browser_harness_is_configured",
            "rationale": harness_prose(alias_loop_fn, alias_slug, alias_chain_words,
                                       command)
                         + text_note(command, text_contains[0],
                                     run_needle_site if command == "run"
                                     else test_needle_site),
            "steps": [harness_step(command, entry, json_output=False, thread_flags=True,
                                   env_var=HARNESS_ENV,
                                   asserts={"stdout_contains":
                                            ["8"] if command == "run" else ["8\nok 1"]})]})
    for command, entry in (("run", "main.${ext}"), ("test", "smoke.test.${ext}")):
        cases.append({
            "name": f"json_{command}_supports_{alias_slug}_when_browser_harness_is_configured",
            "rationale": harness_prose(alias_loop_fn, alias_slug, alias_chain_words,
                                       "--output json " + command)
                         + json_note(command, 0),
            "steps": [harness_step(command, entry, json_output=True, thread_flags=True,
                                   env_var=HARNESS_ENV,
                                   json_claims=harness_json(command, stdout_pin=pin,
                                                            stderr=True, errors=False),
                                   asserts={})]})

    # --- member chain -------------------------------------------------------
    member_slug = "bracketed_global_this_math_pow_member_chain"
    cases.append({
        "name": f"build_emits_{member_slug}",
        "rationale": bundle_prose("member", member_slug, member_bundle_fn,
                                  member_chain_words, rename_note),
        "steps": bundle_steps_for_stem("app_member_chain", "${ext}", member_body,
                                       {"stdout_contains": ["8\n"]},
                                       json_output=False)})
    cases.append({
        "name": f"json_build_emits_{member_slug}",
        "rationale": bundle_prose("member", member_slug, member_bundle_fn,
                                  member_chain_words, rename_note) + json_build_note,
        "steps": bundle_steps_for_stem("app_member_chain", "${ext}", member_body,
                                       {"stdout_contains": ["8\n"]},
                                       json_output=True,
                                       json_claims=envelope_build(errors=False))})
    for command, entry in (("run", "main_member_chain.${ext}"),
                           ("test", "smoke_member_chain.test.${ext}")):
        cases.append({
            "name": f"{command}_supports_{member_slug}_when_browser_harness_is_configured",
            "rationale": harness_prose(member_loop_fn, member_slug, member_chain_words,
                                       command)
                         + text_note(command, text_contains[1],
                                     run_needle_site if command == "run"
                                     else test_needle_site)
                         + rename_note,
            "steps": [harness_step(command, entry, json_output=False, thread_flags=True,
                                   env_var=HARNESS_ENV,
                                   asserts={"stdout_contains":
                                            ["8"] if command == "run" else ["8\nok 1"]})]})
    for command, entry in (("run", "main_member_chain.${ext}"),
                           ("test", "smoke_member_chain.test.${ext}")):
        cases.append({
            "name": f"json_{command}_supports_{member_slug}_when_browser_harness_is_configured",
            "rationale": harness_prose(member_loop_fn, member_slug, member_chain_words,
                                       "--output json " + command)
                         + json_note(command, 1) + rename_note,
            "steps": [harness_step(command, entry, json_output=True, thread_flags=True,
                                   env_var=HARNESS_ENV,
                                   json_claims=harness_json(command, stdout_pin=pin,
                                                            stderr=True, errors=False),
                                   asserts={})]})

    if len(cases) * len(EXTS) != 48:
        raise AssertionError(
            f"rule 7: {len(cases)} cases x ext({len(EXTS)}) != 48 invocations")

    source = {
        "app.${ext}": alias_bundle_src,
        "main.${ext}": alias_run,
        "smoke.test.${ext}": alias_test,
        "app_member_chain.${ext}": member_bundle_src,
        "main_member_chain.${ext}": member_run,
        "smoke_member_chain.test.${ext}": member_test,
    }
    assert_rename_is_argv_only(
        source, ["app_member_chain", "main_member_chain", "smoke_member_chain"])
    if len(set(source.values())) != len(source):
        raise AssertionError("expected six DISTINCT programs; two [source] bodies match")
    return (f"{stem}.toml", header, {"ext": EXTS}, source, cases)


# ==========================================================================
# D5. browser_math_pow_zero_exponent_non_integer_base.rs -- 28 fns, 28
#     invocations, FOUR helpers, NO matrix.
# ==========================================================================
@target("math_pow_zero_exponent_non_integer_base")
def pow_zero_exponent_non_integer_base():
    stem = "math_pow_zero_exponent_non_integer_base"
    text = rs(stem)
    cite = lambda pat, expect=1: P.cite_line(text, pat, expect=expect)  # noqa: E731

    plain_bundle_fn = "assert_browser_bundle_math_pow_zero_exponent_non_integer_base"
    brk_bundle_fn = ("assert_browser_bundle_bracketed_global_this_math_pow_zero_exponent_"
                     "non_integer_base")
    plain_harness_fn = "assert_browser_harness_math_pow_zero_exponent_non_integer_base"
    brk_harness_fn = ("assert_browser_harness_bracketed_global_this_math_pow_zero_"
                      "exponent_non_integer_base")
    plain_bundle_src_fn = "browser_bundle_math_pow_zero_exponent_non_integer_base_source"
    plain_run_fn = "browser_harness_math_pow_zero_exponent_non_integer_base_run_source"
    plain_test_fn = "browser_harness_math_pow_zero_exponent_non_integer_base_test_source"
    brk_bundle_src_fn = ("browser_bundle_bracketed_global_this_math_pow_zero_exponent_"
                         "non_integer_base_source")
    brk_run_fn = ("browser_harness_bracketed_global_this_math_pow_zero_exponent_non_"
                  "integer_base_run_source")
    brk_test_fn = ("browser_harness_bracketed_global_this_math_pow_zero_exponent_non_"
                   "integer_base_test_source")

    plain_bundle_src = check_program("plain bundle", fixture_in_fn(text, plain_bundle_src_fn))
    plain_run = check_program("plain run", fixture_in_fn(text, plain_run_fn))
    plain_test = check_program("plain test", fixture_in_fn(text, plain_test_fn))
    brk_bundle_src = check_program("bracketed bundle", fixture_in_fn(text, brk_bundle_src_fn))
    brk_run = check_program("bracketed run", fixture_in_fn(text, brk_run_fn))
    brk_test = check_program("bracketed test", fixture_in_fn(text, brk_test_fn))
    plain_body = check_program(
        "plain harness body",
        fixture_starting(text, plain_bundle_fn, "const mod = await import("),
        must_contain="await import(")
    brk_body = check_program(
        "bracketed harness body",
        fixture_starting(text, brk_bundle_fn, "const mod = await import("),
        must_contain="await import(")

    # --- ruling 8: the four `_in_json_<ext>_input` fns issue NON-json commands
    # and duplicate their unprefixed counterparts byte for byte. Asserted, not
    # eyeballed, before any prose says so.
    dup_pairs = []
    for command, ext in (("run", "jsx"), ("run", "tsx"), ("test", "jsx"), ("test", "tsx")):
        plain = (f"{command}_supports_math_pow_zero_exponent_non_integer_base_"
                 f"when_browser_harness_is_configured_in_{ext}_input")
        jsonish = (f"{command}_supports_math_pow_zero_exponent_non_integer_base_"
                   f"when_browser_harness_is_configured_in_json_{ext}_input")
        P.assert_identical(f"{jsonish} vs {plain}",
                           fn_body(text, plain), fn_body(text, jsonish))
        dup_pairs.append((jsonish, plain))
    if "json_output" in fn_body(text, plain_harness_fn):
        raise AssertionError(
            f"`{plain_harness_fn}` does have a json_output parameter; the ruling-8 note "
            "would be false")
    if "--output" in fn_body(text, plain_harness_fn):
        raise AssertionError(f"`{plain_harness_fn}` does add --output; note would be false")

    # --- citations ----------------------------------------------------------
    successes = cite(r"output\.status\.success\(\)", expect=6)
    p_build_ok, p_bundle_harness_ok = successes[0], successes[1]
    p_harness_ok = successes[2]
    b_build_ok, b_bundle_harness_ok = successes[3], successes[4]
    b_harness_ok = successes[5]
    env_line = cite(r'assert_eq!\(envelope\["schemaVersion"\], 1\)', expect=2)
    env_fmt = cite(r'payload\["bundleFormat"\]', expect=2)
    meta_a = cite(r'metadata\["apiSurface"\]', expect=2)
    meta_b = cite(r'metadata\["artifactKind"\]', expect=2)
    bundle_contains = cite(r'stdout\.contains\("1\\n1\\n"\)', expect=2)
    harness_contains = cite(r'stdout\.contains\(expected_stdout\)', expect=2)
    env_sites = cite(r"BROWSER_HARNESS_COMMAND_ENV", expect=2)
    thread_sites = cite(r'\.arg\("--max-threads"\)', expect=2)
    # 8 call sites each: 6 through the plain helper, 2 through the bracketed one.
    run_needle_sites = cite(r'^\s+"1\\n1",$', expect=8)
    test_needle_sites = cite(r'^\s+"1\\n1\\nok 1",$', expect=8)
    run_needle_site = [run_needle_sites[0], run_needle_sites[6]]
    test_needle_site = [test_needle_sites[0], test_needle_sites[6]]
    dup_sites = [cite(r"\bfn\s+" + re.escape(name)) for name, _ in dup_pairs]
    orig_sites = [cite(r"\bfn\s+" + re.escape(name)) for _, name in dup_pairs]

    renamed = ["app_bracketed.js", "app_bracketed.ts",
               "main_bracketed.js", "main_bracketed.ts",
               "smoke_bracketed.test.js", "smoke_bracketed.test.ts"]
    header_extra = [P.extra_ok(name, EXTRA_OK_U5_RENAME) for name in renamed]

    decline_reason = para(
        "Four helpers, and they do not share one axis. "
        f"`{plain_bundle_fn}` runs 8 = ext(js/ts/jsx/tsx) x json_output(false/true). "
        f"`{brk_bundle_fn}` runs only 4 = ext(js/ts) x json_output(false/true) -- TWO "
        "extensions, not four. "
        f"`{plain_harness_fn}` runs 12 and `{brk_harness_fn}` runs 4, and neither takes a "
        "`json_output` parameter at all: both take an `expected_stdout` instead and issue "
        "a NON-json command, so those 16 invocations have no JSON sibling to pair with. "
        "An `ext` axis would fan the bracketed helpers to jsx and tsx, which the source "
        "never runs; a `json_output` axis would manufacture JSON envelopes for 16 "
        "invocations that never requested one.")

    ruling8 = []
    for (jsonish, plain), dsite, osite in zip(dup_pairs, dup_sites, orig_sites):
        ruling8 += para(P.migration_note_stale_fn_name(
            jsonish,
            f"the name says `json`, but `{plain_harness_fn}` (:{cite(chr(102) + 'n ' + re.escape(plain_harness_fn))}) "
            f"has no `json_output` parameter, never appends `--output json`, and has no "
            f"json branch, so this fn (:{dsite}) issues a plain-text command. It is "
            f"byte-identical to `{plain}` (:{osite}) -- asserted mechanically in this "
            f"file's generator by comparing the two fn bodies, not eyeballed. Per rule 6 "
            f"it still gets its own `[[case]]`: a case is the only remaining trace of its "
            f"fn, and no JSON envelope claim is written for it, because the source makes "
            f"none.")) + [""]
    ruling8 = ruling8[:-1] if ruling8 else ruling8

    u5 = P.u5_renames([
        ("app.js / app.ts", "app_bracketed.js / app_bracketed.ts",
         "the bracketed bundle program, which the source also writes to `app.<ext>`; the "
         "`file_json` path and the harness `entry` follow the rename, because "
         "`kali build --bundle` names its output directory after the input stem"),
        ("main.js / main.ts", "main_bracketed.js / main_bracketed.ts",
         "the bracketed run program"),
        ("smoke.test.js / smoke.test.ts",
         "smoke_bracketed.test.js / smoke_bracketed.test.ts",
         "the bracketed test program"),
    ])

    assertion_shape = para(
        f"ASSERTION SHAPE, mirrored from the source and nothing more. Both bundle "
        f"helpers: `exit = \"success\"` on the build (:{p_build_ok}, :{b_build_ok}) and "
        f"on the harness process (:{p_bundle_harness_ok}, :{b_bundle_harness_ok}); in "
        f"json mode the envelope's schemaVersion/command/success/exitCode and payload "
        f"artifactKind/bundleFormat (:{env_line[0]}-{env_fmt[0]}, "
        f":{env_line[1]}-{env_fmt[1]}) with NO `errors` claim, so `errors = []` appears "
        f"nowhere in this file; the emitted bundle metadata "
        f"(:{meta_a[0]}-{meta_b[0]}, :{meta_a[1]}-{meta_b[1]}), read outside the "
        f"`if json_output` and so asserted in BOTH modes; then the harness step's "
        f"`stdout.contains(\"1\\n1\\n\")` (:{bundle_contains[0]}, "
        f":{bundle_contains[1]}).") + [""] + para(
        f"Both harness helpers: `exit = \"success\"` (:{p_harness_ok}, :{b_harness_ok}) "
        f"and `stdout.contains(expected_stdout)` (:{harness_contains[0]}, "
        f":{harness_contains[1]}), and NOTHING ELSE. There is no json branch in either "
        f"helper, so no case below carries a `json` envelope claim on a run/test step, "
        f"no `json.stdout` pin, and no `stderr` claim -- writing any of those would "
        f"invent a claim (rule 2). `expected_stdout` is supplied by the call site as "
        f"\"1\\n1\" for `run` (:{run_needle_site[0]} plain, :{run_needle_site[1]} "
        f"bracketed) and \"1\\n1\\nok 1\" for `test` (:{test_needle_site[0]} plain, "
        f":{test_needle_site[1]} bracketed), carried verbatim as each case's "
        f"`stdout_contains` needle.") + [""] + para(
        f"Both harness helpers pass `--max-threads 0 --max-spawned-processes 0` "
        f"(:{thread_sites[0]}, :{thread_sites[1]}) and set the harness command through "
        f"`kali_runtime_contract`'s BROWSER_HARNESS_COMMAND_ENV constant "
        f"(:{env_sites[0]}, :{env_sites[1]}), which resolves to {HARNESS_ENV}; neither "
        f"build argv passes a thread flag or sets an env var. No `.matches(...).count()` "
        f"claim exists anywhere in this file, so no `stdout_count`/`json_count` key "
        f"appears.")

    header = block(
        EXTRA_HEAD + header_extra,
        [f"Migrated from tests/browser_{stem}.rs."],
        P.rule12_no_comments_prose(os.path.join(TESTS, f"browser_{stem}.rs"), stem),
        P.matrix_declined(test_fns=28, invocations=28, cases=28, reason=decline_reason),
        P.RULE6_ONE_TO_ONE,
        P.u2_source_file_wide(
            ["app.<ext>", "main.<ext>", "smoke.test.<ext>", "app_bracketed.<ext>",
             "main_bracketed.<ext>", "smoke_bracketed.test.<ext>"]),
        u5,
        P.RULING7_NO_HOIST,
        P.rule13_header([
            "kali_bin", plain_bundle_src_fn, plain_bundle_fn, plain_run_fn,
            plain_test_fn, plain_harness_fn, brk_bundle_src_fn, brk_bundle_fn,
            brk_run_fn, brk_test_fn, brk_harness_fn]),
        ruling8,
        P.ARGV_ORDER,
        assertion_shape,
    )

    # --- rationale prose ----------------------------------------------------
    plain_words = ("raises a non-integer base of 1.6 to the zero exponent through both a "
                   "bare `Math.pow` and a dotted `globalThis.Math.pow`, so both print `1`")
    brk_words = ("raises a non-integer base of 1.6 to the zero exponent through a "
                 "bracketed `globalThis[\"Math\"].pow` and a fully bracketed "
                 "`globalThis[\"Math\"][\"pow\"]`, so both print `1`")

    def bundle_rationale(fn_name, ext, words, idx, json_output, renamed_note=""):
        text_ = (
            f"Migrated from browser_{stem}.rs, one `#[test]` fn per (ext, output shape) "
            f"cell; `[matrix]` is declined file-wide, so this case is exactly one source "
            f"fn (rule 6). `{fn_name}` builds a browser bundle "
            f"(`kali build --bundle --api browser`), asserts the emitted bundle metadata "
            f"(:{meta_a[idx]}-{meta_b[idx]}), then runs the bundle glue under the "
            f"browser-bundle-harness contract, against a program that {words}. The only "
            f"output claim on that step is `stdout.contains(\"1\\n1\\n\")` "
            f"(:{bundle_contains[idx]}). " + P.ruling3_substring())
        if json_output:
            text_ += (
                f" This sibling asserts the JSON build envelope (schemaVersion/command/"
                f"success/exitCode and payload artifactKind/bundleFormat, "
                f":{env_line[idx]}-{env_fmt[idx]}) instead of plain text; the source "
                f"makes no `errors` claim on it, so none is written (rule 2).")
        return text_ + renamed_note

    def harness_rationale(fn_name, command, words, idx, extra=""):
        needle_site = (run_needle_site if command == "run" else test_needle_site)[idx]
        needle = '"1\\n1"' if command == "run" else '"1\\n1\\nok 1"'
        out = (
            f"Migrated from browser_{stem}.rs, one `#[test]` fn per (command, ext) cell; "
            f"`[matrix]` is declined file-wide, so this case is exactly one source fn "
            f"(rule 6). `{fn_name}` runs `kali {command} --api browser --max-threads 0 "
            f"--max-spawned-processes 0` with the browser harness backed by node "
            f"(BROWSER_HARNESS_COMMAND_ENV, :{env_sites[idx]}), against a program that "
            f"{words}. It asserts exit success (:{p_harness_ok if idx == 0 else b_harness_ok}) "
            f"and `stdout.contains(expected_stdout)` (:{harness_contains[idx]}) and "
            f"nothing else; this call site supplies `expected_stdout` = {needle} "
            f"(:{needle_site}), carried verbatim as the `stdout_contains` needle. "
            + P.ruling3_substring() +
            " The helper takes no `json_output` parameter and never adds `--output json`, "
            "so there is no JSON envelope claim, no `json.stdout` pin and no `stderr` "
            "claim on this case -- the source makes none.")
        return out + extra

    dup_note = (
        " MIGRATION NOTE (controller ruling 8) applies to this case: its source fn is "
        "named `..._in_json_<ext>_input` but issues a plain-text command, and is "
        "byte-identical to the `..._in_<ext>_input` fn above it -- asserted mechanically "
        "in this file's generator. Both are kept as separate cases because a `[[case]]` "
        "is the only remaining trace of its fn (rule 6); the source is not corrected. "
        "See the file header."
    )
    rename_note = (
        " U5: the source writes this program to the same filename its non-bracketed "
        "sibling uses, which one flat file-wide `[source]` table cannot represent, so the "
        "key is variant-suffixed. The name is passed to `kali` on argv only and is "
        "referenced by no fixture body, so the rename does not rewrite the program under "
        "test."
    )

    cases = []
    base = "math_pow_zero_exponent_non_integer_base"
    for ext in EXTS:
        cases.append({
            "name": f"build_emits_{base}_in_{ext}_input",
            "rationale": bundle_rationale(plain_bundle_fn, ext, plain_words, 0, False),
            "steps": bundle_steps(f"app.{ext}", plain_body,
                                  {"stdout_contains": ["1\n1\n"]},
                                  json_output=False, meta_fields=META)})
    for ext in EXTS:
        cases.append({
            "name": f"json_build_emits_{base}_in_{ext}_input",
            "rationale": bundle_rationale(plain_bundle_fn, ext, plain_words, 0, True),
            "steps": bundle_steps(f"app.{ext}", plain_body,
                                  {"stdout_contains": ["1\n1\n"]},
                                  json_output=True,
                                  json_claims=envelope_build(errors=False),
                                  meta_fields=META)})

    def plain_harness_case(command, ext, entry, name_infix="", extra=""):
        return {
            "name": f"{command}_supports_{base}_when_browser_harness_is_configured_"
                    f"in_{name_infix}{ext}_input",
            "rationale": harness_rationale(plain_harness_fn, command, plain_words, 0, extra),
            "steps": [harness_step(command, entry, json_output=False, thread_flags=True,
                                   env_var=HARNESS_ENV,
                                   asserts={"stdout_contains":
                                            ["1\n1"] if command == "run"
                                            else ["1\n1\nok 1"]})]}

    # Source fn order: run js, run ts, test js, test ts, run jsx, run tsx,
    # test jsx, test tsx, then the four `_in_json_<ext>_` duplicates.
    cases.append(plain_harness_case("run", "js", "main.js"))
    cases.append(plain_harness_case("run", "ts", "main.ts"))
    cases.append(plain_harness_case("test", "js", "smoke.test.js"))
    cases.append(plain_harness_case("test", "ts", "smoke.test.ts"))
    cases.append(plain_harness_case("run", "jsx", "main.jsx"))
    cases.append(plain_harness_case("run", "tsx", "main.tsx"))
    cases.append(plain_harness_case("test", "jsx", "smoke.test.jsx"))
    cases.append(plain_harness_case("test", "tsx", "smoke.test.tsx"))
    cases.append(plain_harness_case("run", "jsx", "main.jsx", "json_", dup_note))
    cases.append(plain_harness_case("run", "tsx", "main.tsx", "json_", dup_note))
    cases.append(plain_harness_case("test", "jsx", "smoke.test.jsx", "json_", dup_note))
    cases.append(plain_harness_case("test", "tsx", "smoke.test.tsx", "json_", dup_note))

    brk_base = "bracketed_global_this_math_pow_zero_exponent_non_integer_base"
    for ext in ("js", "ts"):
        cases.append({
            "name": f"build_emits_{brk_base}_in_{ext}_input",
            "rationale": bundle_rationale(brk_bundle_fn, ext, brk_words, 1, False,
                                          rename_note),
            "steps": bundle_steps_for_stem("app_bracketed", ext, brk_body,
                                           {"stdout_contains": ["1\n1\n"]},
                                           json_output=False)})
    for ext in ("js", "ts"):
        cases.append({
            "name": f"json_build_emits_{brk_base}_in_{ext}_input",
            "rationale": bundle_rationale(brk_bundle_fn, ext, brk_words, 1, True,
                                          rename_note),
            "steps": bundle_steps_for_stem("app_bracketed", ext, brk_body,
                                           {"stdout_contains": ["1\n1\n"]},
                                           json_output=True,
                                           json_claims=envelope_build(errors=False))})
    for command, stem_name in (("run", "main_bracketed"), ("test", "smoke_bracketed.test")):
        for ext in ("js", "ts"):
            cases.append({
                "name": f"{command}_supports_{brk_base}_when_browser_harness_is_"
                        f"configured_in_{ext}_input",
                "rationale": harness_rationale(brk_harness_fn, command, brk_words, 1,
                                               rename_note),
                "steps": [harness_step(command, f"{stem_name}.{ext}", json_output=False,
                                       thread_flags=True, env_var=HARNESS_ENV,
                                       asserts={"stdout_contains":
                                                ["1\n1"] if command == "run"
                                                else ["1\n1\nok 1"]})]})

    if len(cases) != 28:
        raise AssertionError(f"rule 6: expected 28 named siblings, built {len(cases)}")

    source = {}
    for ext in EXTS:
        source[f"app.{ext}"] = plain_bundle_src
    for ext in EXTS:
        source[f"main.{ext}"] = plain_run
    for ext in EXTS:
        source[f"smoke.test.{ext}"] = plain_test
    for ext in ("js", "ts"):
        source[f"app_bracketed.{ext}"] = brk_bundle_src
    for ext in ("js", "ts"):
        source[f"main_bracketed.{ext}"] = brk_run
    for ext in ("js", "ts"):
        source[f"smoke_bracketed.test.{ext}"] = brk_test
    # Ruling 7's mandatory half: every duplicated body is provably ONE extracted
    # program repeated, not two texts that merely look alike.
    P.assert_identical("plain bundle body", *[source[f"app.{e}"] for e in EXTS])
    P.assert_identical("plain run body", *[source[f"main.{e}"] for e in EXTS])
    P.assert_identical("plain test body", *[source[f"smoke.test.{e}"] for e in EXTS])
    P.assert_identical("bracketed bundle body",
                       *[source[f"app_bracketed.{e}"] for e in ("js", "ts")])
    P.assert_identical("bracketed run body",
                       *[source[f"main_bracketed.{e}"] for e in ("js", "ts")])
    P.assert_identical("bracketed test body",
                       *[source[f"smoke_bracketed.test.{e}"] for e in ("js", "ts")])
    assert_rename_is_argv_only(
        source, ["app_bracketed", "main_bracketed", "smoke_bracketed"])
    return (f"{stem}.toml", header, None, source, cases)


def main(argv):
    assert_env_constant()
    names = argv or sorted(REGISTRY)
    for name in names:
        if name not in REGISTRY:
            raise SystemExit(f"unknown target {name!r}; known: {sorted(REGISTRY)}")
        out, header, matrix, source, cases = REGISTRY[name]()
        write(os.path.join(CASES, out), emit(header, matrix, source, cases))


if __name__ == "__main__":
    main(sys.argv[1:])
