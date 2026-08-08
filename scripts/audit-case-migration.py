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
    arguments, whichever side of the comma the literal is on, found via a
    balanced-paren, string-aware argument scanner (`_assert_eq_literal_
    tokens`), not a regex -- see the FIXED note above `ASSERT_EQ_VALUE_FIRST`
    's old definition for why a regex could not do this safely. Site counts
    quoted anywhere in this project's history for "assert_eq! vs .contains()"
    have come from at least three different measurement tools and disagreed
    each time; the only number worth trusting is one reproducible from the
    code actually shipped here, so: running the exact CONTAINS pattern and
    `_assert_eq_literal_tokens` scanner below over crates/kali_cli/tests/*.rs
    (2026-08-07, PRE-fix, against the file set present that day, most of
    which has since migrated away and cannot be re-measured on the same
    corpus) found 1,744 assert_eq! string-literal sites (via the old regex
    pair, all value-second; value-first was 0 that day) against 1,229
    .contains() literal sites -- assert_eq! is the dominant assertion form,
    not .contains(). Do not requote this pair from memory; if it is ever in
    question again, re-run `CONTAINS`/`_assert_eq_literal_tokens` against
    `Path('crates/kali_cli/tests').glob('*.rs')` as it stands then -- the
    file set itself is a moving target across this migration project, so a
    fresh re-run will not reproduce 1,744/1,229 exactly even with identical
    code, and that is expected, not a regression. A migration that keeps a
    JSON path (`errors.0.code`) but silently asserts a different value
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
`stdout_absent`, `stderr`, `stderr_contains`, `stderr_absent`, every string leaf and
every key inside `json`/`fields`, and `[constants]` values (referenced into
assertions via `${NAME}`, so a rule constant vanishing from `[constants]`
matters exactly like it did in the old `const NAME: &str` form). Both the
inline single-step shorthand and `[[case.step]]` lists are read. `name`,
`rationale`, `ignore`, `kind`, and `path`/`entry` carry no claim (they are
file references), so they don't affect assertions. `body` and everything
under `[source]` are program text, not claims about behavior. `exit` asserts
exit status -- a real assertion, but not a string literal, so it's out of
scope for a literal-coverage audit specifically. `matrix` is axis data: its
values are substituted into `args`/`stdout_contains`/etc. via `${...}` before
assertions are read (`crates/kali_case_runner/src/expand.rs`), the same way
`[constants]` values are, so a matrix value's claim is audited in the field
it substitutes into, not at its own declaration site. Together, these are
excluded *by construction* -- there's no set of keys left to enumerate them
out of, so a sixth prose home can't quietly appear the way a sixth spelling
could keep appearing against a blacklist. A claim that exists only in
`rationale`/a comment/a case name/a fixture is correctly reported missing: a
value that matters belongs in an assertion, not next to one.

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
# assert_eq!(lhs, "literal") / assert_eq!("literal", rhs) — literal as either
# argument. Extracted by `_assert_eq_literal_tokens` below (a balanced-paren,
# string-aware argument scanner), NOT a `[^,]*`-skip regex.
#
# FIXED (found during Task 17, same class as the `JSON_KEY` raw-string
# anchor and the `unquote()` unicode-escape fix below: the tool's own
# canonicalization/extraction was incomplete, not the migrated file's
# assertion): the previous `ASSERT_EQ_VALUE_SECOND` was
# `assert_eq!\(\s*[^,]*,\s*(LITERAL)\s*[,)]`. Its first-argument placeholder,
# `[^,]*`, does not track parenthesis depth, so on a first argument that
# itself contains a nested call with its own comma-separated arguments --
# `assert_eq!(run_js(&src.replace("var x = 1;", "var x = 2;")), "v=100\n")`
# (`switch_runtime.rs`) -- it stops at the FIRST comma it meets textually,
# which is `.replace`'s own internal separator, not `assert_eq!`'s top-level
# one, and then reads the next string literal (`"var x = 2;"`, a replacement
# ARGUMENT, not an assertion value) as if it were `assert_eq!`'s real second
# argument. That phantom claim lives only in fixture-construction text, which
# is exactly the kind of text `[source]` excludes from the new side's search
# by design (see the module docstring above), so nothing but a fabricated
# assertion could ever satisfy it -- the audit failed on `switch_runtime.rs`
# for a claim that was never a claim. The old docstring called the `[^,]*`
# design "deliberately conservative" and accepted a false NEGATIVE on a
# first-argument top-level comma as its cost; it did not anticipate a
# first-argument NESTED comma producing a false POSITIVE instead, which is
# the strictly more dangerous direction (a false OK is worse than a missed
# claim). Confirmed additive-and-corrective, not just additive, by re-running
# every currently-migrated pair's audit before and after this fix (see the
# task-17 report): every previously-`AUDIT OK` file stayed `AUDIT OK` with an
# unchanged claim set (the old regex's bare-second-argument case is a special
# case the new scanner also handles identically), and `switch_runtime.rs`
# flipped from `AUDIT FAILED` (2 phantom claims) to `AUDIT OK` with zero
# newly-missing real claims.
#
# `CONTAINS`/`CONST`/`JSON_KEY`/`ARG` above and below do NOT share this bug:
# none of them has a `[^,]*`-style "skip an unbounded prefix" component --
# each requires its captured literal to start immediately (mod whitespace/an
# optional `&`) at a fixed anchor point (right after `.contains(`, right
# after `const NAME: &str =`, right after `[`, right after `.arg(`), so there
# is nothing for a nested call's comma to be mistaken for. Re-verified
# empirically, not just by inspection: the same before/after sweep that
# re-ran every pair's full audit (all five claim kinds, not just assert_eq
# values) found zero changes to any `contains literals`/`rule constants`/
# `json keys`/`argv tokens` claim set anywhere in the corpus.
# assert_eq!(json["a"]["b"], value) — capture each bracketed key.
JSON_KEY = re.compile(r'\[\s*"([A-Za-z_][A-Za-z0-9_]*)"\s*\]')
# .arg("token")
ARG = re.compile(r'\.arg\(\s*"([^"]*)"\s*\)')
TEST_FN = re.compile(r'#\[test\][^\n]*\n(?:\s*#\[[^\]]*\]\s*\n)*\s*(?:async\s+)?fn\s+([a-z0-9_]+)')

# A raw-string literal (`r"..."`, `r#"..."#`, `r##"..."##`, ...), fence-aware
# via a backreference on the captured `#` run so a raw string's genuinely
# unescaped interior quotes -- routine in a JS/TS fixture body, e.g.
# `globalThis["String"]["fromCharCode"]` -- don't prematurely end the match.
# `re.DOTALL` lets `.` cross the newlines every multi-line fixture body has.
#
# `(?<![A-Za-z0-9_])` in front of the `r` is load-bearing, not decorative:
# without it, this pattern fires on *any* `r` immediately preceding a `"`,
# including the last letter of an ordinary word inside a *plain* string
# literal that happens to end in `r` -- `"unsupported operato r"` reads as
# `...operato` + a spurious raw-string open at the final `r"`. That
# spuriously-opened "raw string" then runs until the next `"` + matching
# `#`-count it finds, which is very often the *next* real literal in the
# file -- e.g. in `assert!(stderr.contains("unsupported
# operator")); assert_eq!(json["errors"][0]["code"], "E5506");`, it
# consumes through to the `"` opening `"errors"`, blanking the `["` of
# `json["errors"]` and silently dropping that key from the audit. Measured
# directly against all 307 files in crates/kali_cli/tests/*.rs before this
# anchor existed: 1,509 spurious raw-string matches, 93 real JSON keys lost
# across 92 files (mostly `stderr`, plus e.g. `artifactKind` in
# `browser_find_family_bundle.rs`) -- i.e. an unanchored version of this
# fix reintroduces a strictly *larger* instance of the exact false-negative
# failure mode it exists to close (see `_blank_raw_strings`'s doc comment
# on why a false negative here is worse than the false positive being
# fixed). The lookbehind requires the character before `r` to be anything
# other than an identifier character (or nothing, i.e. start-of-string),
# which is exactly "a new token is starting here" for every real raw-string
# literal in this corpus, and is what closes the false match above (the
# `r` in `operator` is preceded by `o`, an identifier character, so the
# lookbehind rejects it).
#
# Known residual, not fixed here: `(?:.*?)` still cannot tell a genuine
# `r#"` token start from the same three characters appearing inside a line
# comment, a block comment, or the interior of an unrelated plain string --
# this is a regex approximation, not a real Rust lexer. Not present
# anywhere in the corpus at time of writing; acceptable for that reason,
# not because it is impossible in principle.
_RAW_STRING = re.compile(r'(?<![A-Za-z0-9_])r(#*)"(?:.*?)"\1', re.DOTALL)


def _blank_raw_strings(source: str) -> str:
    """`source` with every raw-string literal's entire span (delimiters and
    interior alike) replaced by spaces of the same length. Used only to
    build a search text for `JSON_KEY`, so that JS/TS fixture source --
    always written in this corpus as a raw string precisely because a raw
    string's interior can hold unescaped quotes (a
    `fn supported_source() -> &'static str { r#"..."# }` body, an
    `fs::write(&path, r#"..."#)` argument) -- cannot masquerade as a real
    `json["key"]`/`envelope["key"]`/`harness["key"]` assertion. Confirmed
    concretely: `string_from_char_code_static_ascii.rs`'s fixture contains
    `globalThis["String"]["fromCharCode"](65)` inside an `r#"..."#` body,
    which the unmasked `JSON_KEY` regex reads as two JSON-path claims
    ("String", "fromCharCode") that no case file could ever satisfy short
    of fabricating an assertion, because `[source]` is deliberately excluded
    from the new side's claim search (see this script's module docstring).

    Only raw strings need masking. A *plain* (non-raw) Rust string literal
    used for the same purpose must escape any embedded quote as `\\"` (two
    characters: backslash then quote) -- and `JSON_KEY`'s
    `\\[\\s*"ident"\\s*\\]` requires a bare `"` immediately after `[`, so an
    escaped `\\"` never matches it to begin with. `string_search.rs`'s
    plain-string fixture (`"console.log(\\"hello\\".includes(...))"`) is
    confirmed harmless this way without any masking, and a genuine
    top-level claim like `"schemaVersion"` in `json["schemaVersion"]` is
    itself a plain string (no `r` prefix), so this function never touches
    it either.

    An identifier-allowlist approach (only count `["key"]` when it
    immediately follows a known JSON-value receiver name, e.g. `json`) was
    considered and rejected: the receiver name is not a small closed set in
    this corpus today (`json`, `envelope`, `metadata`, `payload`,
    `contract`, `meta`, `harness`, `value`, `test_json`, `source_map`, ... —
    confirmed by grepping crates/kali_cli/tests, not hypothetical), and a
    real claim can be reached with no `let NAME = ...` binding to discover
    at all -- e.g. `array_concat_static.rs`'s
    `json["errors"].as_array()...iter().any(|error| { error["code"] ==
    "E5506" && ... })`, where `error` is a closure parameter. Enumerating
    receivers would have silently under-counted that claim (a false
    negative -- worse than the false positive being fixed here). Masking
    keys off *where the text lives* (inside vs. outside a raw-string span),
    which is what actually distinguishes fixture source from real Rust
    assertion code, and needs no knowledge of receiver names at all.

    That "where the text lives" judgment is only as correct as `_RAW_STRING`'s
    own left anchor. Getting the span boundary wrong is not a cosmetic bug:
    an over-eager span swallows real code (and the JSON keys in it) the same
    way an unmasked fixture swallows a phantom claim, just in the opposite,
    more dangerous direction -- a false negative on this side means the
    audit reports OK while a real claim silently vanished. See the
    `(?<![A-Za-z0-9_])` comment on `_RAW_STRING` above for the concrete case
    that was previously getting this wrong.
    """
    return _RAW_STRING.sub(lambda m: " " * len(m.group(0)), source)


def _skip_string(text: str, pos: int) -> int | None:
    """If `text[pos]` opens a string literal (plain `"..."` or raw
    `r#*"..."#*`), return the index one past its closing delimiter;
    otherwise `None`. A raw-string open is only recognized when `pos` is a
    genuine token start (the character before it, if any, is not an
    identifier character) -- the same guard `_RAW_STRING` above needs and
    for the identical reason (an ordinary word ending in `r`, e.g.
    `"...operator")`, must not be misread as the start of a raw string)."""
    n = len(text)
    c = text[pos]
    if c == '"':
        j = pos + 1
        while j < n:
            if text[j] == '\\':
                j += 2
                continue
            if text[j] == '"':
                return j + 1
            j += 1
        return n
    if c == 'r' and (pos == 0 or not (text[pos - 1].isalnum() or text[pos - 1] == '_')):
        k = pos + 1
        hashes = 0
        while k < n and text[k] == '#':
            hashes += 1
            k += 1
        if k < n and text[k] == '"':
            close = '"' + ('#' * hashes)
            end = text.find(close, k + 1)
            return (end + len(close)) if end != -1 else n
    return None


def _split_top_level_args(arg_text: str) -> list[str]:
    """Split a call's raw argument-list text (the text strictly between its
    outer parens) into top-level arguments: a comma only splits when it is
    not inside a nested `()`/`[]`/`{}` and not inside a string literal. This
    is what `_find_calls`/`_assert_eq_literal_tokens` use in place of the
    former `[^,]*` regex skip -- see the FIXED note above `ASSERT_EQ_VALUE
    _FIRST`'s old definition for the false-positive this closes.

    Bracket TYPES are not matched against each other (an unclosed `(` can be
    closed by a `]`) -- this is a heuristic scan over syntactically valid
    Rust, the same trade-off `callsite_extract.py`-style tools on this
    branch have made before, not a real parser. It is sufficient here: the
    source is always valid Rust, so bracket types are always self-consistent
    even though this function does not check that itself.
    """
    parts = []
    depth = 0
    start = 0
    i = 0
    n = len(arg_text)
    while i < n:
        c = arg_text[i]
        if c == '"' or c == 'r':
            end = _skip_string(arg_text, i)
            if end is not None:
                i = end
                continue
        if c in '([{':
            depth += 1
        elif c in ')]}':
            depth -= 1
        elif c == ',' and depth == 0:
            parts.append(arg_text[start:i])
            start = i + 1
        i += 1
    parts.append(arg_text[start:])
    return [p.strip() for p in parts]


def _find_calls(source: str, name: str) -> list[str]:
    """Every balanced `name(...)` call's raw argument-list text (the text
    strictly between the outer parens), found by depth-tracking from the
    opening paren to its true matching close -- skipping string-literal
    interiors the same way `_split_top_level_args` does, so a `)` or `(`
    character inside a fixture string can never be mistaken for the call's
    own closing paren. `name` is matched literally (e.g. `"assert_eq!("`
    already includes the macro's own opening paren in the search anchor is
    NOT assumed here -- callers pass the bare name, e.g. `"assert_eq!"`)."""
    out = []
    n = len(source)
    for m in re.finditer(re.escape(name) + r'\(', source):
        depth = 1
        i = m.end()
        while i < n and depth > 0:
            c = source[i]
            if c == '"' or c == 'r':
                end = _skip_string(source, i)
                if end is not None:
                    i = end
                    continue
            if c in '([{':
                depth += 1
            elif c in ')]}':
                depth -= 1
                if depth == 0:
                    break
            i += 1
        out.append(source[m.end():i])
    return out


def _assert_eq_literal_tokens(source: str) -> list[str]:
    """Every literal-string argument in position 0 or 1 of every
    `assert_eq!(...)` call, found via balanced-paren, string-aware argument
    splitting -- see the FIXED note above for the bug this replaces. Returns
    raw literal tokens (quotes and escapes intact), exactly like
    `CONTAINS`/`CONST`'s `.findall()` output, so it flows through the same
    `unquote()`/`literal_variants()` pipeline unchanged. A 3rd+ argument (a
    custom panic-message format string) is never inspected, matching the
    old regexes' scope."""
    tokens = []
    for arg_text in _find_calls(source, 'assert_eq!'):
        for a in _split_top_level_args(arg_text)[:2]:
            if re.fullmatch(_STR_LITERAL, a):
                tokens.append(a)
    return tokens


class _AssertEqLiterals:
    """Duck-types `re.Pattern`'s `.findall(source) -> list[str]` so
    `LITERAL_KINDS` can hold this scanner alongside real compiled regexes
    without changing `claims()`'s iteration logic at all."""

    def findall(self, source: str) -> list[str]:
        return _assert_eq_literal_tokens(source)


ASSERT_EQ_LITERALS = _AssertEqLiterals()


# String-literal claim kinds, each checked in both spellings (see module
# docstring), each backed by one or more patterns whose matches are unioned.
LITERAL_KINDS: dict[str, list] = {
    "contains literals": [CONTAINS],
    "rule constants": [CONST],
    "assert_eq values": [ASSERT_EQ_LITERALS],
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


# A Rust `\u{XXXX}` unicode escape (1-6 hex digits), not preceded by another
# backslash (so an already-escaped literal backslash-u, i.e. `\\u{...}` in the
# source meaning a literal `\` followed by the four characters `u{...}`, is
# left alone rather than double-decoded -- not observed in this corpus at
# time of writing, but cheap to guard against). `unquote()` was missing this
# case entirely: `"6\nh\u{e9}llo"` canonicalized to text that still contained
# the literal six characters `\`, `u`, `{`, `e`, `9`, `}`, so it could never
# match a migrated case's real, correct UTF-8 assertion (`héllo`) -- a false
# "claim missing" that would have forced choosing between fabricating a wrong
# assertion (the literal escape sequence, which is not what the program
# prints) and leaving a real claim unaudited. This mirrors the `JSON_KEY`
# raw-string anchor fix and the `json_null` key sync: the tool's
# canonicalization was incomplete, not the migrated file's assertion.
_UNICODE_ESCAPE = re.compile(r'(?<!\\)\\u\{([0-9a-fA-F]{1,6})\}')


def unquote(raw: str) -> str:
    """Turn a Rust string literal token into its fully-unescaped text.

    `_UNICODE_ESCAPE` must run *before* the plain `\\\\` -> `\\` collapse, not
    after. Its `(?<!\\)` guard exists so a genuine escaped-backslash-then-
    literal-text token -- Rust source `"\\\\u{e9}"`, meaning a literal
    backslash followed by the five literal characters `u{e9}` -- is left
    alone rather than decoded as if it were the real unicode escape
    `"\\u{e9}"` (a single backslash + `u{e9}`, meaning U+00E9). Applying the
    `\\\\` -> `\\` collapse first destroys that distinction: it collapses the
    double backslash down to one *before* `_UNICODE_ESCAPE` ever sees the
    text, so the guard's lookbehind finds no preceding backslash and wrongly
    decodes the collapsed text to `é`. Running `_UNICODE_ESCAPE` on the
    pre-collapse text instead lets the lookbehind see the real second
    backslash and correctly skip it; the later `\\\\` -> `\\` collapse then
    reduces the untouched double backslash to the single literal backslash
    the source actually meant. Inert on this corpus at the time of this fix
    (no `.rs` file contains the literal `\\u{` byte pattern that would
    trigger it), verified additive by re-running this script against all 50
    then-migrated `.rs`/`.toml` pairs before and after: 50/50 both times.
    """
    raw = raw.strip()
    if raw.startswith("r"):
        raw = raw[1:]
        hashes = len(raw) - len(raw.lstrip("#"))
        return raw[hashes + 1 : len(raw) - hashes - 1]
    body = raw[1:-1]
    body = body.replace('\\"', '"').replace("\\n", "\n").replace("\\t", "\t")
    body = _UNICODE_ESCAPE.sub(lambda m: chr(int(m.group(1), 16)), body)
    return body.replace("\\\\", "\\")


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

    for key in JSON_KEY.findall(_blank_raw_strings(source)):
        out["json keys"][key] = frozenset({key})

    for tok in ARG.findall(source):
        out["argv tokens"][tok] = frozenset({tok})

    return out


# The keys the case runner (crates/kali_case_runner/src/model.rs) actually
# turns into assertions, on a single resolved Step. Anything not named here
# — name, rationale, ignore, kind, path, entry (file references, not claims),
# matrix (axis data, audited in the fields it substitutes into), body and all
# of [source] (program text), exit (non-literal assertion: exit status) — is
# excluded by never being read, not by being pattern-matched away. Keep this
# in sync with `Step`/`RawStep` in model.rs; a field added there that carries
# a string-literal claim (a new assertion key) needs a line here too.
_STEP_LIST_KEYS = (
    "args",
    "stdout_contains",
    "stdout_absent",
    "stderr_contains",
    "stderr_absent",
    "json_null",
)
_STEP_SCALAR_KEYS = ("stdout", "stderr")
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
