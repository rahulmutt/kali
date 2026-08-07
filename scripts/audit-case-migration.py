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
  - `assert_eq!(a, "literal")` / `assert_eq!("literal", a)` string-literal
    arguments, whichever side of the comma the literal is on. Site counts
    quoted anywhere in this project's history for "assert_eq! vs .contains()"
    have come from at least three different measurement tools and disagreed
    each time; the only number worth trusting is one reproducible from the
    code actually shipped here, so: running the exact CONTAINS,
    ASSERT_EQ_VALUE_SECOND, and ASSERT_EQ_VALUE_FIRST patterns below over
    crates/kali_cli/tests/*.rs (2026-08-07) finds 1,744 assert_eq!
    string-literal sites (all value-second; value-first is 0 today, see its
    comment) against 1,229 .contains() literal sites -- assert_eq! is the
    dominant assertion form, not .contains(). Reproduce with:
    `python3 -c "..."` using this module's CONTAINS/ASSERT_EQ_VALUE_*
    patterns against `Path('crates/kali_cli/tests').glob('*.rs')` if this
    number is ever in question again -- do not requote it from memory.
    A migration that keeps a JSON path (`errors.0.code`) but silently
    asserts a different value ("E5507" instead of "E5506") is exactly the
    kind of quiet weakening this script exists to catch, and only this
    claim kind catches it.
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

The new case files are never searched as raw text. A blacklist approach
(strip known prose homes -- comments, `rationale`) was tried first and
proved to be whack-a-mole: `rationale = \"\"\"...\"\"\"` (TOML's other
triple-quote form -- and the one the design spec's own worked example
uses), an inline trailing `# ...` comment, a case `name` that happens to
contain the diagnostic code, and a `// ...` comment inside a `[source]`
fixture body are four more homes a literal can hide in, none of them
stripped, all four verified to independently produce a false "every claim
present" on a case file where every real `json.errors.0.code` assertion had
been changed to the wrong diagnostic. Three of those four are things a
careful author does with no intent to cheat: following the spec's own
example, naming a case after the rule it pins, commenting a fixture. A gate
that a documentation habit can satisfy produces false confidence at exactly
the moment someone is being thorough, which is worse than no gate.

So this parses each new case file with `tomllib` (stdlib) and only *searches
the fields the case runner (`kali_case_runner::model`) actually turns into
assertions*: a step's `args`, `env` values, `stdout`, `stdout_contains`,
`stdout_absent`, `stderr_contains`, `stderr_absent`, every string leaf and
every key inside `json`/`fields`, and `[constants]` values (referenced into
assertions via `${NAME}`, so a rule constant vanishing from `[constants]`
matters exactly like it did in the old `const NAME: &str` form). Both the
inline single-step shorthand and `[[case.step]]` lists are read. `name`,
`rationale`, `ignore`, `kind`, `path`/`entry` (file references, not claims),
and everything under `[source]` (program text, not an assertion about
program behavior) are excluded *by construction* -- there's no set of keys
left to enumerate them out of, so a sixth prose home can't quietly appear
the way a sixth spelling could keep appearing against a blacklist. A claim
that exists only in `rationale`/a comment/a case name/a fixture is correctly
reported missing: a value that matters belongs in an assertion, not next to
one.

A useful side effect of parsing instead of pattern-matching: `tomllib`
resolves `"a\nb"` and a `'''`/`\"\"\"` literal block to the identical Python
string, so the two-spellings-of-a-newline problem that motivated this
script's own dual-form matching (below) does not recur on the new-file
side -- it's solved once, correctly, by using a real parser, rather than
solved approximately per spelling by pattern-matching raw text.

This is a coverage check, not a proof of equivalence. It catches wholesale drops
and quiet weakenings (a rule constant vanishing while `contains("E5506")`
survives). It cannot catch a claim that was rewritten to be weaker while keeping
its literals. Read the diff too.

Usage: audit-case-migration.py OLD.rs NEW.toml [NEW.toml ...]
"""

from __future__ import annotations

import re
import sys
import tomllib
from pathlib import Path

_STR_LITERAL = r'r?#*"(?:[^"\\]|\\.)*"#*'

# Literal arguments to .contains(...) — one of several dominant assertion forms.
CONTAINS = re.compile(rf'\.contains\(\s*(?:&)?({_STR_LITERAL})')
# const NAME: &str = "literal";
CONST = re.compile(rf'const\s+[A-Z0-9_]+\s*:\s*&str\s*=\s*\n?\s*({_STR_LITERAL})')
# assert_eq!(lhs, "literal") — literal as the second argument, the shape this
# corpus actually uses today.
# The first argument is captured with a bare [^,]* (no comma allowed) rather
# than a real expression parser: this is deliberately conservative. It correctly
# skips calls whose first argument itself contains a top-level comma (a nested
# multi-arg call, a vec! literal, ...) rather than risk mis-splitting them, at
# the cost of a false negative for that (rare, and always non-string-adjacent
# in this corpus at time of writing) shape. It matches across newlines, so
# multi-line assert_eq!(...) calls are still captured.
ASSERT_EQ_VALUE_SECOND = re.compile(rf'assert_eq!\(\s*[^,]*,\s*({_STR_LITERAL})\s*[,)]')
# assert_eq!("literal", rhs) — literal as the first argument. No site of this
# shape exists in the corpus at time of writing (confirmed), but nothing
# guarantees that stays true across ~300 more migrations, and missing it
# would be a silent under-extraction, not a loud one -- so both argument
# positions are covered rather than only the one currently observed.
ASSERT_EQ_VALUE_FIRST = re.compile(rf'assert_eq!\(\s*({_STR_LITERAL})\s*,')
# assert_eq!(json["a"]["b"], value) — capture each bracketed key.
JSON_KEY = re.compile(r'\[\s*"([A-Za-z_][A-Za-z0-9_]*)"\s*\]')
# .arg("token")
ARG = re.compile(r'\.arg\(\s*"([^"]*)"\s*\)')
TEST_FN = re.compile(r'#\[test\][^\n]*\n(?:\s*#\[[^\]]*\]\s*\n)*\s*(?:async\s+)?fn\s+([a-z0-9_]+)')

# String-literal claim kinds, each checked in both spellings (see module
# docstring), each backed by one or more patterns whose matches are unioned.
LITERAL_KINDS: dict[str, list[re.Pattern[str]]] = {
    "contains literals": [CONTAINS],
    "rule constants": [CONST],
    "assert_eq values": [ASSERT_EQ_VALUE_SECOND, ASSERT_EQ_VALUE_FIRST],
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

    for kind, patterns in LITERAL_KINDS.items():
        bucket = out[kind]
        for pattern in patterns:
            for token in pattern.findall(source):
                canonical = unquote(token)
                bucket[canonical] = bucket.get(canonical, frozenset()) | literal_variants(token)

    for key in JSON_KEY.findall(source):
        out["json keys"][key] = frozenset({key})

    for tok in ARG.findall(source):
        out["argv tokens"][tok] = frozenset({tok})

    return out


# The keys the case runner (crates/kali_case_runner/src/model.rs) actually
# turns into assertions, on a single resolved Step. Anything not named here
# — name, rationale, ignore, kind, path, entry, and all of [source] — is
# excluded by never being read, not by being pattern-matched away. Keep this
# in sync with `Step`/`RawStep` in model.rs; a field added there that
# carries a literal claim (a new assertion key) needs a line here too.
_STEP_LIST_KEYS = ("args", "stdout_contains", "stdout_absent", "stderr_contains", "stderr_absent")
_STEP_SCALAR_KEYS = ("stdout",)
_STEP_JSON_KEYS = ("json", "fields")
# Keys inside a case's non-step namespace (name/rationale/ignore/step) that
# are never assertion-bearing and must not be treated as the inline step.
_CASE_NON_STEP_KEYS = frozenset({"name", "rationale", "ignore", "step"})
# A TOML table key shaped like an array index (see model.rs's dotted-path
# jsonpath doc comment) carries no claim of its own -- consistent with the
# old JSON_KEY regex, which only ever matched identifier-shaped keys.
_IDENTIFIER_KEY = re.compile(r'[A-Za-z_][A-Za-z0-9_]*')


def _json_like_strings(value: object) -> list[str]:
    """Every string leaf, and every identifier-shaped key, in a parsed
    `json`/`fields` value -- both are part of the claim (see module
    docstring: "every string leaf and every key")."""
    out: list[str] = []
    if isinstance(value, dict):
        for key, sub in value.items():
            if isinstance(key, str) and _IDENTIFIER_KEY.fullmatch(key):
                out.append(key)
            out.extend(_json_like_strings(sub))
    elif isinstance(value, list):
        for sub in value:
            out.extend(_json_like_strings(sub))
    elif isinstance(value, str):
        out.append(value)
    return out


def _step_assertion_strings(step: dict) -> list[str]:
    """The claim-bearing strings on one resolved step (inline or from
    `[[case.step]]`), reading only the whitelisted keys."""
    out: list[str] = []
    for key in _STEP_LIST_KEYS:
        out.extend(v for v in step.get(key, []) or [] if isinstance(v, str))
    for key in _STEP_SCALAR_KEYS:
        value = step.get(key)
        if isinstance(value, str):
            out.append(value)
    env = step.get("env")
    if isinstance(env, dict):
        out.extend(v for v in env.values() if isinstance(v, str))
    for key in _STEP_JSON_KEYS:
        if key in step:
            out.extend(_json_like_strings(step[key]))
    return out


def assertion_strings(doc: dict) -> list[str]:
    """Every claim-bearing string in one parsed case file: `[constants]`
    values, plus each case's inline step and/or `[[case.step]]` list."""
    out: list[str] = []

    constants = doc.get("constants")
    if isinstance(constants, dict):
        out.extend(v for v in constants.values() if isinstance(v, str))

    for case in doc.get("case", []) or []:
        if not isinstance(case, dict):
            continue
        steps: list[dict] = []
        inline = {k: v for k, v in case.items() if k not in _CASE_NON_STEP_KEYS}
        if inline:
            steps.append(inline)
        step_list = case.get("step")
        if isinstance(step_list, list):
            steps.extend(s for s in step_list if isinstance(s, dict))
        for step in steps:
            out.extend(_step_assertion_strings(step))

    return out


def load_new_text(paths: list[Path]) -> str:
    """Parse every new case file and return only its assertion-bearing
    strings, joined for substring search. See the module docstring for why
    this is a parse, not a text search over the raw file."""
    pieces: list[str] = []
    for path in paths:
        try:
            doc = tomllib.loads(path.read_text())
        except tomllib.TOMLDecodeError as error:
            print(f"error: {path}: invalid TOML: {error}", file=sys.stderr)
            raise SystemExit(2) from error
        pieces.extend(assertion_strings(doc))
    return "\n".join(pieces)


def main() -> int:
    if len(sys.argv) < 3:
        print(__doc__, file=sys.stderr)
        return 2

    old_path = Path(sys.argv[1])
    new_paths = [Path(p) for p in sys.argv[2:]]

    old_source = old_path.read_text()
    new_text = load_new_text(new_paths)

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
