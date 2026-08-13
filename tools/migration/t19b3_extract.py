#!/usr/bin/env python3
r"""Shape-strict extractor for Task 19 batch 3's `run_source` family.

WHAT THIS IS AND WHY IT EXISTS (U12). All seven of batch 3's targets share ONE
mechanical shape: a file-local

    fn run_source(src: &str) -> std::process::Output

that writes exactly one program to `main.ts` under a fresh temp directory and
runs `kali run <path>` on it, plus N `#[test]` fns that each call it exactly
once. 123 `#[test]` fns across the seven. Hand-listing 123 case specs is how a
dropped claim gets shipped, so the claim set of every case is DERIVED from the
source here instead -- and, far more importantly:

  **EVERY `assert!`/`assert_eq!` IN A TEST BODY MUST MATCH A RECOGNISED SHAPE
  OR THIS RAISES.**

That is the whole design. A forward extractor that silently skips what it does
not understand is the shape this project keeps finding: it turns a dropped
assertion into a green run. `claims_of` therefore enumerates every assertion
macro in the body, matches each against the closed shape table below, and
raises `UnknownShape` naming the file, the fn and the verbatim condition when
one does not match. Adding a source with a new assertion shape breaks the
generator; it cannot quietly under-claim.

THE SHAPE TABLE (closed; anything else raises):

  assert!(out.status.success(), ...)         -> exit = "success"
  assert!(!out.status.success(), ...)        -> exit = "failure"
  assert_eq!(String::from_utf8_lossy(&out.stdout), <str>)
                                             -> stdout = <str>   (exact pin)
  assert!(out.stdout.is_empty(), ...)        -> stdout = ""      (exact pin)
  assert!(<stderr-expr>.contains(<str>), ...)-> stderr_contains += <str>
  assert!(<stderr-expr>.contains(A) || <stderr-expr>.contains(B), ...)
                                             -> rule 11 / ruling 17 disjunction

`<stderr-expr>` is `String::from_utf8_lossy(&out.stderr)` or a `let`-binding of
it; the binding is resolved, not pattern-matched by name.

WHAT IS DELIBERATELY NOT A CLAIM. The second and later arguments of an
`assert!` are its PANIC MESSAGE -- `"stderr: {}"`, `"must reject"` -- which the
program under test never sees. They are not assertions and are not migrated,
which is also how `audit-case-migration.py` and batch 2's fidelity accounting
treat them.

FIXTURE RESOLUTION (rules 8/9). `fixture_of` returns the literal the test
actually hands to `run_source`, resolved one of exactly two ways: the call's
argument is a string literal, or it is an identifier bound by a `let <id> = <string
literal>;` earlier in the same body. Anything else raises. Nothing selects by
position and nothing is retyped, so a drifting line cannot pick the wrong
program, and a source that starts building its fixture some third way fails
here rather than shipping a program that is not the program under test.

  Usage:
    t19b3_extract.py            # census over the seven targets; rc=1 on any
                                # unrecognised shape
    t19b3_extract.py <stem>     # one target, verbose
"""

from __future__ import annotations

import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, os.path.join(REPO, "tools/task-18-browser-pilot"))

from comment_coverage import (extract_comment_paragraphs,  # noqa: E402
                              extract_trailing_comments, is_divider)
from lexer import find_string_literals  # noqa: E402

TESTS = os.path.join(REPO, "crates/kali_cli/tests")

# The seven targets, and the ONE predicate that selected them. Kept here rather
# than in the generator because `--census` has to be runnable on its own.
STEMS = [
    "param_compound_assign",
    "runtime_join",
    "runtime_module_globals",
    "runtime_string_arrays",
    "runtime_string_value_flow",
    "runtime_substring_length",
    "runtime_ternary",
]

RUN_SOURCE_SIG = "fn run_source(src: &str) -> std::process::Output {"


class UnknownShape(AssertionError):
    """An assertion in a test body that this extractor does not model.

    Raised rather than skipped. See the module docstring: a forward extractor
    that skips what it does not understand converts a dropped claim into a
    green run.
    """


# --------------------------------------------------------------------------
# Lexical helpers
# --------------------------------------------------------------------------

def _blank_line_comments(text: str) -> str:
    """`text` with `//` comment bodies replaced by spaces, string-aware.

    A commented-out `assert!` is not an assertion, and a `//` inside a string
    literal is not a comment. Both directions matter here: the first because a
    commented-out claim must not become a case claim, the second because these
    fixtures are full of `"http://"`-shaped and `//`-carrying program text.

    Newlines are preserved so every offset in the returned string still names
    the same line as in the original.
    """
    spans = [(l["start"], l["end"]) for l in find_string_literals(text)]
    out = list(text)
    i, n = 0, len(text)
    si = 0
    while i < n:
        while si < len(spans) and spans[si][1] <= i:
            si += 1
        if si < len(spans) and spans[si][0] <= i < spans[si][1]:
            i = spans[si][1]
            continue
        if text[i] == "/" and i + 1 < n and text[i + 1] == "/":
            j = text.find("\n", i)
            j = n if j == -1 else j
            for k in range(i, j):
                out[k] = " "
            i = j
            continue
        i += 1
    return "".join(out)


def _balanced(text: str, open_idx: int) -> int:
    """Index just past the `)` matching the `(` at `open_idx`, string-aware."""
    spans = [(l["start"], l["end"]) for l in find_string_literals(text[open_idx:])]
    spans = [(a + open_idx, b + open_idx) for a, b in spans]
    depth, i, n = 0, open_idx, len(text)
    si = 0
    while i < n:
        while si < len(spans) and spans[si][1] <= i:
            si += 1
        if si < len(spans) and spans[si][0] <= i < spans[si][1]:
            i = spans[si][1]
            continue
        if text[i] == "(":
            depth += 1
        elif text[i] == ")":
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    raise AssertionError("unbalanced parentheses")


def _split_top_commas(text: str) -> list[str]:
    """Split on commas at paren/bracket/brace depth 0, string-aware."""
    spans = [(l["start"], l["end"]) for l in find_string_literals(text)]
    parts, cur, depth, i, n, si = [], [], 0, 0, len(text), 0
    while i < n:
        while si < len(spans) and spans[si][1] <= i:
            si += 1
        if si < len(spans) and spans[si][0] <= i < spans[si][1]:
            cur.append(text[i:spans[si][1]])
            i = spans[si][1]
            continue
        c = text[i]
        if c in "([{":
            depth += 1
        elif c in ")]}":
            depth -= 1
        if c == "," and depth == 0:
            parts.append("".join(cur))
            cur = []
            i += 1
            continue
        cur.append(c)
        i += 1
    parts.append("".join(cur))
    return [p.strip() for p in parts]


def _one_literal(frag: str, *, where: str) -> str:
    lits = find_string_literals(frag)
    if len(lits) != 1:
        raise UnknownShape(f"{where}: expected exactly one string literal in "
                           f"{frag.strip()!r}, found {len(lits)}")
    return lits[0]["value"]


def _norm(s: str) -> str:
    return re.sub(r"\s+", " ", s).strip()


# --------------------------------------------------------------------------
# Test-fn enumeration
# --------------------------------------------------------------------------

def _fn_body(text: str, brace: int) -> tuple[int, int]:
    spans = [(l["start"], l["end"]) for l in find_string_literals(text[brace:])]
    spans = [(a + brace, b + brace) for a, b in spans]
    depth, i, n, si = 0, brace, len(text), 0
    while i < n:
        while si < len(spans) and spans[si][1] <= i:
            si += 1
        if si < len(spans) and spans[si][0] <= i < spans[si][1]:
            i = spans[si][1]
            continue
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                return brace, i + 1
        i += 1
    raise AssertionError("unbalanced braces")


def test_fns(text: str) -> list[dict]:
    """`[{name, body, start_line, attr_line}]` for every `#[test]` fn."""
    out = []
    masked = _blank_line_comments(text)
    for m in re.finditer(r"#\[test\]\s*\nfn\s+([a-z_0-9]+)\s*\(\s*\)\s*\{", masked):
        brace = m.end() - 1
        lo, hi = _fn_body(text, brace)
        out.append({
            "name": m.group(1),
            "body": text[lo:hi],
            "masked_body": masked[lo:hi],
            "attr_line": text.count("\n", 0, m.start()) + 1,
            "start_line": text.count("\n", 0, brace) + 1,
        })
    return out


# --------------------------------------------------------------------------
# Fixture
# --------------------------------------------------------------------------

def fixture_of(stem: str, fn: dict) -> str:
    """The program text this test hands to `run_source`, resolved not retyped."""
    body, masked = fn["body"], fn["masked_body"]
    where = f"{stem}.rs::{fn['name']}"
    calls = [m for m in re.finditer(r"\brun_source\s*\(", masked)]
    if len(calls) != 1:
        raise UnknownShape(f"{where}: {len(calls)} `run_source(` call(s), wanted 1")
    open_idx = calls[0].end() - 1
    end = _balanced(body, open_idx)
    arg = body[open_idx + 1:end - 1].strip().rstrip(",").strip()
    lits = find_string_literals(arg)
    if len(lits) == 1 and _norm(arg).startswith(('"', 'r"', 'r#')):
        return lits[0]["value"]
    if re.fullmatch(r"[a-z_][a-z_0-9]*", arg):
        binding = re.search(r"\blet\s+" + re.escape(arg) + r"\s*(?::\s*&str\s*)?=\s*",
                            masked)
        if not binding:
            raise UnknownShape(
                f"{where}: `run_source({arg})` but no `let {arg} = ...` in the body")
        tail = body[binding.end():]
        lits = find_string_literals(tail)
        if not lits or lits[0]["start"] != 0:
            raise UnknownShape(
                f"{where}: `let {arg} = ...` is not bound directly to a string literal")
        return lits[0]["value"]
    raise UnknownShape(
        f"{where}: `run_source(...)` argument is neither a string literal nor a "
        f"`let`-bound one: {_norm(arg)[:80]!r}")


# --------------------------------------------------------------------------
# Claims
# --------------------------------------------------------------------------

_STDERR_EXPR = r"(?:String::from_utf8_lossy\(\s*&\s*out\.stderr\s*\)|stderr)"
_STDOUT_LOSSY = r"String::from_utf8_lossy\(\s*&\s*out\.stdout\s*\)"

_SUCCESS = re.compile(r"^out\.status\.success\(\)$")
_FAILURE = re.compile(r"^!\s*out\.status\.success\(\)$")
_STDOUT_EMPTY = re.compile(r"^out\.stdout\.is_empty\(\)$")
_CONTAINS = re.compile(r"^" + _STDERR_EXPR + r"\.contains\((.+)\)$")


def _macro_calls(masked_body: str, body: str) -> list[tuple[str, str]]:
    """`(macro, argtext)` for every `assert!`/`assert_eq!`/`assert_ne!` call."""
    out = []
    for m in re.finditer(r"\b(assert_eq!|assert_ne!|assert!)\s*\(", masked_body):
        open_idx = m.end() - 1
        end = _balanced(body, open_idx)
        out.append((m.group(1), body[open_idx + 1:end - 1]))
    return out


def claims_of(stem: str, fn: dict, *, computed_stdout=None) -> dict:
    """The claim set of one `#[test]` fn, or `UnknownShape`.

    `computed_stdout` supplies the value for the ONE test in this batch whose
    expected stdout is computed in Rust and therefore exists as no literal
    (`runtime_module_globals::module_var_lcg_float_division`). Passing it is how
    that case is admitted; without it the shape raises, which is the correct
    default -- a pin with no literal behind it must be declared, never inferred.
    """
    where = f"{stem}.rs::{fn['name']}"
    body, masked = fn["body"], fn["masked_body"]
    claims = {"exit": None, "stdout": None, "stderr_contains": [],
              "disjunctions": [], "stdout_source": None}

    # `let stderr = String::from_utf8_lossy(&out.stderr);` -- resolved, so the
    # `.contains` shapes below do not have to know what the binding is called.
    stderr_bindings = set(re.findall(
        r"\blet\s+([a-z_][a-z_0-9]*)\s*=\s*String::from_utf8_lossy\(\s*&\s*out\.stderr\s*\)",
        masked))

    def is_stderr_recv(recv: str) -> bool:
        recv = _norm(recv)
        return (recv in stderr_bindings
                or re.fullmatch(r"String::from_utf8_lossy\(\s*&\s*out\.stderr\s*\)", recv)
                is not None)

    for macro, argtext in _macro_calls(masked, body):
        args = _split_top_commas(argtext)
        cond = _norm(args[0])
        if macro == "assert_ne!":
            raise UnknownShape(f"{where}: `assert_ne!` is not a modelled shape")
        if macro == "assert_eq!":
            lhs, rhs = _norm(args[0]), _norm(args[1])
            if not re.fullmatch(_STDOUT_LOSSY, lhs):
                raise UnknownShape(
                    f"{where}: `assert_eq!` whose left side is not stdout: {lhs!r}")
            lits = find_string_literals(rhs)
            if len(lits) == 1 and lits[0]["start"] == 0 and lits[0]["end"] == len(rhs):
                value, provenance = lits[0]["value"], "copied"
            elif re.fullmatch(r"[a-z_][a-z_0-9]*", rhs):
                bind = re.search(r"\blet\s+" + re.escape(rhs) + r"\s*=\s*", masked)
                if not bind:
                    raise UnknownShape(f"{where}: `assert_eq!` against unbound {rhs!r}")
                tail = body[bind.end():]
                lits = find_string_literals(tail)
                if lits and lits[0]["start"] == 0:
                    value, provenance = lits[0]["value"], "copied"
                elif computed_stdout is not None:
                    value, provenance = computed_stdout, "computed"
                else:
                    raise UnknownShape(
                        f"{where}: `let {rhs} = ...` is COMPUTED in Rust, so the "
                        "expected stdout exists as no literal. Supply it as a "
                        "declared capture (`computed_stdout=`) -- it must not be "
                        "inferred.")
            else:
                raise UnknownShape(
                    f"{where}: `assert_eq!` right side is neither a literal nor a "
                    f"`let`-bound name: {rhs[:80]!r}")
            if claims["stdout"] is not None and claims["stdout"] != value:
                raise UnknownShape(f"{where}: two different exact stdout pins")
            claims["stdout"] = value
            claims["stdout_source"] = provenance
            continue
        # assert!
        if _SUCCESS.fullmatch(cond):
            claims["exit"] = "success"
            continue
        if _FAILURE.fullmatch(cond):
            claims["exit"] = "failure"
            continue
        if _STDOUT_EMPTY.fullmatch(cond):
            if claims["stdout"] not in (None, ""):
                raise UnknownShape(f"{where}: `stdout.is_empty()` beside an exact pin")
            claims["stdout"] = ""
            claims["stdout_source"] = "is_empty"
            continue
        # `A.contains(x)` / `A.contains(x) || B.contains(y)`
        alts = [a.strip() for a in re.split(r"\|\|", cond)]
        parsed = []
        for alt in alts:
            m = re.fullmatch(r"(.+?)\.contains\((.+)\)", alt)
            if not m or not is_stderr_recv(m.group(1)):
                parsed = None
                break
            parsed.append(_one_literal(m.group(2), where=where))
        if parsed is None:
            raise UnknownShape(f"{where}: unmodelled assert condition: {cond[:120]!r}")
        if len(parsed) == 1:
            claims["stderr_contains"].append(parsed[0])
        else:
            claims["disjunctions"].append((cond, parsed))
        continue

    if claims["exit"] is None:
        raise UnknownShape(f"{where}: no exit-status assertion found")
    return claims


# --------------------------------------------------------------------------
# Prose (rule 12 / U6)
# --------------------------------------------------------------------------

def _is_divider_line(line: str) -> bool:
    return re.fullmatch(r"[-=]{3,}", line.strip()) is not None


def _is_wrapped_banner(line: str) -> bool:
    """`---- Final-review Finding 2: ... ----` -- a one-line section banner.

    The sources spell a section two ways: a paragraph BRACKETED by all-dash
    lines, and a single line WRAPPED in dashes. Both are the source saying "a
    section starts here", and rule 12 treats section prose the same either way,
    so both are recognised. Recognising only the first attributed a wrapped
    banner to the one `#[test]` it happened to abut, which is under-attribution
    -- the banner introduces the whole group.
    """
    s = line.strip()
    return bool(re.match(r"^[-=]{3,}\s", s) and re.search(r"\s[-=]{3,}$", s))


def prose(stem: str, text: str) -> dict:
    """Rule-12 attribution for a `run_source` source, DERIVED from position.

    Three populations, and the split is the source's own layout rather than a
    judgement made per file:

      * `per_fn[name]`   -- a comment paragraph (or `///` doc block) that lies
                            INSIDE that `#[test]`'s body, or that directly abuts
                            it (ends on the line above its `#[test]` attribute or
                            above its `fn`). Rule 12: prose attached to a helper
                            goes into the rationale of the cases it reaches; a
                            per-test comment reaches exactly one case. The
                            inside-the-body arm is checked FIRST and is not
                            optional -- `runtime_module_globals`'s two
                            in-body comments and `runtime_join`'s fasta note
                            live there, and an abutting-only predicate filed
                            them as file-wide, which is under-attribution.
      * `sections`       -- `[(banner_text, [fn names])]`. A SECTION intro is any
                            paragraph that sits BETWEEN tests without abutting
                            one: either dash-delimited (bracketed by `----` lines,
                            or a single line wrapped in them) or free-standing
                            with a blank line before the next `#[test]`. Rule 12
                            puts it in the rationale of every case in its section,
                            which runs from it to the next section intro or to end
                            of file. U6's ceiling is respected by construction --
                            a section's prose reaches its own cases and no others.
                            The free-standing arm matters: `param_compound_assign`
                            introduces its five indirect-array tests with an
                            undelimited paragraph, and a dashes-only predicate
                            filed it as file-wide, which under-attributes prose
                            rule 12 says belongs in those five rationales.
      * `file_wide`      -- prose that precedes the FIRST `#[test]`: the `//!`
                            module doc and the comments on `run_source`/`kali_bin`,
                            which every case reaches equally. Ruling 6's exemption
                            is applied to the harness helper's doc (the runner now
                            owns the temp-directory job), so it is carried in the
                            `#` header rather than replicated into 123 rationales.

    Trailing comments (U16) are attributed by line to whichever `#[test]` body
    encloses them, and to `file_wide` when none does.
    """
    fns = test_fns(text)
    spans = [(f["attr_line"], f["start_line"], f["name"]) for f in fns]
    end_of = {f["name"]: f["start_line"] + f["body"].count("\n") for f in fns}
    body_of = {f["name"]: (f["start_line"], end_of[f["name"]]) for f in fns}

    per_fn = {f["name"]: [] for f in fns}
    file_wide = []
    sections = []

    paras = [(n, p) for n, p in extract_comment_paragraphs(text) if not is_divider(p)]
    for start, para in paras:
        last = start + len(para) - 1
        banner = bool(_is_divider_line(para[0]) or _is_divider_line(para[-1])
                      or _is_wrapped_banner(para[0]))
        inside = next((nm for nm, (lo, hi) in body_of.items() if lo <= start <= hi), None)
        abuts = next(
            (nm for a, _s, nm in spans if last == a - 1 or last == a - 2), None)
        before_first = bool(spans) and last < spans[0][0] - 2
        text_lines = [l for l in para if not _is_divider_line(l)]
        block = "\n".join(text_lines).strip("\n")
        if inside is not None:
            per_fn[inside].append((start, block))
        elif before_first or not spans:
            file_wide.append((start, block))
        elif banner or abuts is None:
            reached = [nm for a, _s, nm in spans if a > last]
            sections.append((start, block, reached))
        else:
            per_fn[abuts].append((start, block))

    for line, body in extract_trailing_comments(text):
        owner = next((nm for a, s, nm in spans if s <= line < end_of[nm]), None)
        if owner is not None:
            per_fn[owner].append((line, body))
        else:
            file_wide.append((line, body))

    # A section banner's reach is truncated by the NEXT banner.
    trimmed = []
    for i, (start, block, reached) in enumerate(sections):
        stop = sections[i + 1][0] if i + 1 < len(sections) else 10 ** 9
        keep = [nm for a, _s, nm in spans if start < a < stop and nm in reached]
        trimmed.append((block, keep))
    return {"per_fn": {k: [b for _l, b in sorted(v)] for k, v in per_fn.items()},
            "sections": trimmed,
            "file_wide": [b for _l, b in sorted(file_wide)]}


# --------------------------------------------------------------------------
# Census
# --------------------------------------------------------------------------

def source(stem: str) -> str:
    return open(os.path.join(TESTS, stem + ".rs"), encoding="utf-8").read()


COMPUTED = {
    # The one Rust-computed expectation in the batch. Declared here, and the
    # generator re-derives it by EXECUTING the source's own arithmetic before
    # using it (rule 8: never hand-simulate; run the real code).
    ("runtime_module_globals", "module_var_lcg_float_division"): None,
}


def census(stems=None, *, verbose=False) -> int:
    stems = stems or STEMS
    bad = 0
    tot_fns = tot_claims = 0
    for stem in stems:
        text = source(stem)
        if RUN_SOURCE_SIG not in text:
            print(f"  {stem}: NOT a `run_source` target")
            bad += 1
            continue
        fns = test_fns(text)
        n_attr = len(re.findall(r"#\[test\]", text))
        if len(fns) != n_attr:
            print(f"  {stem}: {n_attr} `#[test]` attributes but {len(fns)} parsed fns")
            bad += 1
        pins = rejects = stderr_claims = disj = 0
        for f in fns:
            try:
                fixture_of(stem, f)
                key = (stem, f["name"])
                c = claims_of(stem, f,
                              computed_stdout="<declared>" if key in COMPUTED else None)
            except UnknownShape as e:
                print(f"  UNKNOWN SHAPE: {e}")
                bad += 1
                continue
            tot_claims += (1 + (c["stdout"] is not None) + len(c["stderr_contains"])
                           + len(c["disjunctions"]))
            pins += c["stdout"] is not None
            rejects += c["exit"] == "failure"
            stderr_claims += len(c["stderr_contains"])
            disj += len(c["disjunctions"])
            if verbose:
                print(f"    {f['name']:<62} exit={c['exit']:<7} "
                      f"stdout={'PIN' if c['stdout'] is not None else '-':<3} "
                      f"stderr_contains={len(c['stderr_contains'])} "
                      f"or={len(c['disjunctions'])}")
        tot_fns += len(fns)
        print(f"  {stem:<28} {len(fns):>3} fn(s)  {pins:>3} exact stdout pin(s)  "
              f"{rejects:>3} reject(s)  {stderr_claims} stderr_contains  {disj} OR")
    print(f"\n{len(stems)} target(s), {tot_fns} `#[test]` fn(s), {tot_claims} claim(s)")
    if bad:
        print(f"EXTRACTOR CENSUS FAILED -- {bad} problem(s)")
        return 1
    print("EXTRACTOR CENSUS OK -- every assertion in every test body matched a "
          "modelled shape")
    return 0


if __name__ == "__main__":
    args = [a for a in sys.argv[1:] if not a.startswith("-")]
    sys.exit(census(args or None, verbose="-v" in sys.argv[1:] or bool(args)))
