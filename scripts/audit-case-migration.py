#!/usr/bin/env python3
"""Fail if a migrated case file drops a claim its .rs predecessor made.

Migrating ~200k lines of assertions is where meaning gets silently dropped, and
this repository has already had two fail-closed tests degrade to asserting
nothing. So the migration gate is mechanical, not eyeballed: every string
literal the old test compared against, every JSON path it asserted on, and every
argv token it passed must still appear somewhere in the new case files.

Six claim kinds are extracted:
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
  - Occurrence-count needles: the `"literal"` in a
    `.matches("literal").count()` asserted inside an `assert!`/`assert_eq!`
    -- the claim shape the `stdout_count`/`json_count` case keys exist to
    carry. Both spellings are read (`assert!(x.matches(L).count() >= K)` and
    `assert_eq!(x.matches(L).count(), K)`), on the raw-stdout branch and on
    the `json["stdout"].as_str()` branch alike, since the arm is anchored on
    `.matches(...).count()` itself and not on the surface it is taken
    against. `.matches(some_variable).count()` yields no literal and so no
    claim -- there is nothing auditable in it. A `.matches("lit").count()`
    OUTSIDE an assertion (e.g. `repeat_n(v, src.matches("console.log(")
    .count())`, live in `browser_math_pow_exponent_one.rs`) is deliberately
    not a claim: it is fixture arithmetic, and reading it as one would
    manufacture a phantom claim no case file could satisfy.

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
every key inside `json`/`fields`, every `stdout_count`/`json_count` claim's
`needle` (and a `json_count`'s `path`), and `[constants]` values (referenced into
assertions via `${NAME}`, so a rule constant vanishing from `[constants]`
matters exactly like it did in the old `const NAME: &str` form). **Only a
REFERENCED constant**: an entry expansion can never reach is excluded from
the search surface and reported as a failure in its own right, because the
search is by substring and a dead constant was otherwise a free-text channel
that could return a genuinely dropped assertion to `AUDIT OK`. See
`unreferenced_constants` for the reproduction and for what "referenced"
means (it is derived from `expand.rs`, not from where the text happens to
sit). Both the
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

TWO DIRECTIONS, DELIBERATELY ASYMMETRIC. Every claim kind above is checked in
the "nothing was dropped" direction: an old literal must appear somewhere in
the new files. `stdout_count`/`json_count` are checked in the *opposite*
direction as well -- every count claim a case file makes must correspond to a
real `.matches("lit").count()` assertion in the old source, with the SAME
needle and the SAME bound (`at_least = 2` against `count() >= 2`,
`exact = 6` against `count() == 6` / `assert_eq!(...count(), 6)`), and a
`json_count`'s `path` segments must be JSON keys the old source indexed.
ONE EXCEPTION, and its bound is the whole of it: a `json_count` whose bound is
exactly `at_least = 1` may be justified by a source `.contains(needle)` instead
of a counting site, because ruling 3's amended clause 4 makes that the mandated
migration of a plain `.contains` against a json string leaf and `>= 1` carries
no number to be unfaithful about. Every other bound still requires a real
counting site. See the acceptance in `count_claim_correspondence`. This
is the only direction that can see a count claim that was invented, mis-
needled, or mis-bounded, and without it the keys are unauditable: a needle
that appears nowhere in the source, carrying a bound the source never states,
otherwise exits `AUDIT OK` (verified on a real clean pair before this arm
existed -- see the task-18 batch-4 report §2c).

Why the reverse check is scoped to the count keys and not applied to every
key: an `exact` bound is a fidelity claim about a NUMBER, and a number cannot
be found by a substring search of the new text, so the count keys are the one
place where forward literal coverage provably proves nothing. The other keys'
extra-direction is left to review and to `cargo test`.

Why the FORWARD direction for count claims is literal coverage (the needle
must appear somewhere in the case files) rather than "an old count claim must
become a count key with a matching bound": measured against the real corpus,
the strict form flags two already-shipped, legitimately-STRONGER migrations --
`browser_bundle_toplevel_start.rs`'s `assert_eq!(stdout.matches("3\\n")
.count(), 1)` and `math_inverse_trig_identities.rs`'s
`assert!(stdout.matches("0\\n").count() >= 3)` were both migrated to an exact
whole-`stdout` equality (`stdout = "3\\n"`, `stdout = "0\\n0\\n0\\n"`), which
implies the count claim and then some. A literal-coverage tool cannot see that
implication, so the strict form would report two true migrations as failures.
The residual this leaves -- a count claim silently downgraded to a bare
`stdout_contains` of the same needle passes, because the needle is present
either way -- is not left silent: every source count claim that no case count
claim reproduces prints a `NOT MIRRORED` line for the reader to disposition.
Advisory, deliberately, for the two-legitimate-migrations reason above; the
task-18 audit-count report names it as the residual it is.

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

# RAW-STRING-AWARE, CAPTURE-GROUP-FREE, AND HASH-COUNT-MATCHED. One member of
# the recogniser class `inst2_probes.probe_raw_string_recogniser_class`
# enumerates; closed in Task 19 batch 4 fix round 1 together with `unquote` and
# `raw_body` below, which is the pairing that makes it safe (see their comment).
#
# THE THREE THINGS THE OLD `r?#*"(?:[^"\\]|\\.)*"#*` GOT WRONG:
#   * it never matched the CLOSING hash count, so `r#"{ "a": 1 }"#` stopped at
#     the first interior quote and the extracted claim was `'r#"{ "'`;
#   * it did not admit the `b`/`c` of a byte or C raw string;
#   * it had no left word boundary, so the trailing `r` of an ordinary word
#     could open one.
#
# CAPTURE-GROUP-FREE IS LOAD-BEARING, and it is why the hash counts are
# ENUMERATED rather than backreferenced. Every caller wraps this in its own
# group -- `CONTAINS`, `CONST`, `COUNT_NEEDLES` all spell `({_STR_LITERAL})` --
# and `findall` returns TUPLES the moment a second group exists, which makes
# `unquote()` die on `'tuple' object has no attribute 'strip'`. A first attempt
# at this fix used `(?P<h>#*)…(?P=h)` and produced exactly that: 185 of 268
# pairs "moved" in the corpus differential, every one of them the audit script
# CRASHING rather than a verdict changing. The number was an artifact of the
# patch, not a property of the corpus.
#
# `_MAX_RAW_HASHES` is a bound on an enumeration, not a claim about the corpus;
# `inst2_probes` section 11 carries the arm that fails if the corpus ever
# exceeds it, so the bound cannot go stale silently.
_MAX_RAW_HASHES = 8
_STR_LITERAL = "(?:" + "|".join(
    [rf'(?<![A-Za-z0-9_])(?:br|cr|r)#{{{n}}}"(?:(?!"#{{{n}}}).)*"#{{{n}}}'
     for n in range(_MAX_RAW_HASHES, -1, -1)]
    + [r'"(?:[^"\\]|\\.)*"']) + ")"

# FIXED (Minor 3, found during Task 17 review): `_STR_LITERAL`'s `\\.`
# alternative matches an escaped character generically -- including the Rust
# string-continuation escape `\<newline>` (a backslash immediately followed
# by a real newline, legal inside a plain string literal and used by
# `switch_runtime.rs`'s `const S`/`const SS` declarations, both wrapped
# across several lines this way). `.` does not match a literal newline
# without `re.DOTALL`, so any pattern built from `_STR_LITERAL` and compiled
# without that flag silently fails to match a continuation-wrapped literal at
# all -- not a false claim, a false ABSENCE of a match, which is why
# `switch_runtime.rs`'s audit reported `rule constants: 0` even though the
# brief's own `grep -c 'const [A-Z0-9_]*:'` finds 2. Harmless on that
# specific file (both `S`/`SS` are fixture text, not rule literals), but the
# brief's own gate ("the rule-constant count must match the const count")
# would have been satisfied only by coincidence on any file whose
# continuation-wrapped `const` genuinely was a rule literal. `re.DOTALL` on
# every pattern built from `_STR_LITERAL` (and the one place `_STR_LITERAL`
# is used directly via `re.fullmatch`, in `_assert_eq_literal_tokens` below)
# closes this for all three literal-extraction paths, not just `CONST`.
_STR_LITERAL_FLAGS = re.DOTALL

# The opener `unquote`/`raw_body` decide raw-ness with. Spelled once so the two
# cannot drift apart, and so it is greppable as a member of the recogniser class.
_RAW_LITERAL_OPEN = re.compile(r'^(?:br|cr|r)(#*)"')

# Literal arguments to .contains(...) — one of several dominant assertion forms.
CONTAINS = re.compile(rf'\.contains\(\s*(?:&)?({_STR_LITERAL})', _STR_LITERAL_FLAGS)
# const NAME: &str = "literal";
CONST = re.compile(rf'const\s+[A-Z0-9_]+\s*:\s*&str\s*=\s*\n?\s*({_STR_LITERAL})', _STR_LITERAL_FLAGS)
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

# `#[path = "sibling_dir/child.rs"] mod name;` and plain `mod name;` /
# `pub mod name;` -- the submodule shapes this corpus uses to split a test
# target across multiple files. A file using either shape declares fewer
# (often zero) top-level #[test] fns and pulls the rest in from sibling
# files; `grep -c '#\[test\]'` on the top-level file alone silently drops
# every test that lives in a submodule. FIXED (Task 18 pilot review round
# 1, finding 5; broadened in round 2; hardened in round 3 after two more
# re-reviews): this script originally resolved only `#[path]`-annotated
# mods, one level deep, from the single file named on argv[1]. Real corpus
# chains defeated that at every stage:
#   - `browser_cdp_smoke.rs` reaches 14 more #[test] fns through a PLAIN
#     `mod cdp_driver;` (round 1's fix printed "1 #[test] fns" / "AUDIT OK",
#     examining 1 of 15 real tests).
#   - `inprocess.rs` reaches its CDP driver through a SECOND level of `mod`
#     nesting (`#[path]`-loaded file -> plain `mod cdp_driver;` -> that
#     file's own two `#[path]` mods) that one-level-deep resolution never
#     followed.
#   - (round 3) `PLAIN_MOD`, run against raw source text, matched `mod x;`
#     appearing inside a comment, a doc comment, or a string/raw-string
#     literal -- LIVE in this corpus: `inprocess/cdp_driver.rs`'s own `//!`
#     doc comment says "...resolve its unqualified `mod cdp_driver;`...",
#     which the unmasked regex read as a real (phantom) declaration. Only
#     harmless there because the phantom name happened to already be
#     visited; a `//!` comment naming a module that does NOT exist would
#     have been a hard, blocking exit-2 error on a legitimate migration.
#     `PLAIN_MOD` was also unanchored on its left side, so `submod x;`
#     would have matched as if it were `mod x;`.
# All of the above are fixed: both mod shapes are resolved, recursively,
# from source text that has every comment and string/char literal masked
# out first (see `_mask_comments_and_strings`), with a left word-boundary
# guard on `mod` itself, and with a visited-set of `.resolve()`d (truly
# canonical, `..`-collapsed) paths so a self-referential or mutually-
# referential mod graph -- including one spelled through a `..`-climbing
# `#[path]` that never lexically repeats -- terminates instead of hanging
# or growing an unbounded path string.
#
# The `#[path]` attribute's string is ALWAYS a path relative to the
# CONTAINING FILE'S OWN DIRECTORY (confirmed against `browser_math_atan2_
# bracketed_root.rs`'s `#[path = "browser_math_atan2_bracketed_root/run.rs"]
# mod run;`, resolved relative to `crates/kali_cli/tests/`; and against
# `inprocess/cdp_driver.rs`'s `#[path = "../cdp_driver/driver.rs"]`, which
# only resolves to the real file when taken relative to `cdp_driver.rs`'s
# OWN directory, `tests/inprocess/`, regardless of how `cdp_driver.rs`
# itself was loaded -- `#[path]` is Rust's escape hatch specifically FROM
# the nesting convention below, so it is never subject to it), not a Rust
# module path.
#
# A PLAIN `mod name;` (no `#[path]`), by contrast, IS subject to Rust's
# real directory-nesting convention, which this script now implements
# rather than approximates with one rule for every file:
#   - A "directory-style" module -- the top-level file named on argv[1]
#     (a crate/binary root), a `#[path]`-loaded file, or a `name/mod.rs`
#     file -- has its OWN plain-mod children resolve `child.rs` or
#     `child/mod.rs` relative to ITS OWN directory (confirmed:
#     `browser_cdp_smoke.rs` -> `cdp_driver/mod.rs`; that file's own
#     `mod driver;`/`mod protocol;` -> `cdp_driver/driver.rs`/
#     `cdp_driver/protocol.rs`, siblings of `mod.rs` itself, not a further
#     subdirectory).
#   - A "leaf-style" module -- an ordinary `name.rs` sibling file found via
#     a PLAIN `mod name;` from ITS OWN including file (i.e., NOT found via
#     `#[path]` and NOT a `mod.rs`) -- has its OWN plain-mod children
#     resolve relative to a SUBDIRECTORY named after itself
#     (`name/child.rs` or `name/child/mod.rs`), matching real Rust
#     semantics for a normally-loaded non-`mod.rs` module file.
#     Unexercised by any file in this corpus at fix time (no flat sibling
#     file here further nests a plain `mod`), but load-bearing for safety:
#     this corpus's submodule names are generic (`run.rs`, `build.rs`,
#     `check.rs`, `test.rs`, `misc.rs`), so a same-named sibling one
#     directory up is a realistic coincidence, and treating every plain
#     mod as directory-style (the round-2 approximation) would have
#     silently resolved such a case to the WRONG, unrelated file --
#     folding foreign claims into the audit, the dangerous direction.
PATH_MOD = re.compile(
    r'#\[path\s*=\s*"([^"]+)"\]'
    r'(?:\s*#\[[^\]]*\])*'  # an intervening attribute, e.g. #[cfg(test)]
    r'\s*(?:pub(?:\([^)]*\))?\s+)?(?<![A-Za-z0-9_])mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;',
)
# A plain (no `#[path]`) `mod name;` / `pub mod name;` declaration, with the
# same intervening-attribute tolerance. Matched separately from PATH_MOD and
# reconciled by masking (see `_find_mod_declarations`) rather than by one
# combined regex, so neither pattern can accidentally swallow the other's
# match or mis-pair a `#[path]` string with the wrong `mod` name.
#
# `(?<![A-Za-z0-9_])` immediately before the literal `mod` (round 3 fix): a
# plain identifier-char lookbehind, so `submod x;` (or any identifier ending
# in "mod") is correctly rejected -- without it, "mod" is matched as a
# substring of any longer word ending the same way.
PLAIN_MOD = re.compile(
    r'(?:#\[[^\]]*\]\s*)*(?:pub(?:\([^)]*\))?\s+)?(?<![A-Za-z0-9_])mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;',
)


def _mask_comments_outside_strings(source: str) -> str:
    """`source` with every real comment blanked (newlines and offsets
    preserved) and every string/char literal left INTACT -- a single
    left-to-right pass that recognizes whichever token starts first, so a
    `//` inside a string literal is text, not a comment.

    This REPLACES a non-string-aware predecessor (`_mask_comments`, deleted
    in the Task 18 audit-count fix round rather than left around to be
    reached for again). That version scanned for `//` and `/*` without
    tracking string literals, and this corpus breaks it twice over, in both
    of its callers:

    - `cdp_driver/driver.rs` asserts
      `assert!(ws_url.starts_with("ws://"), ...)`, whose `//` is inside a
      string. Blanking from there to end-of-line takes the literal's closing
      quote with it, and every subsequent `_skip_string` then runs from an
      unterminated string -- one `assert!` call's argument text swallowed
      14,561 characters of unrelated code, re-minting a
      `.matches("3\\n").count()` claim from an assertion 70 lines further
      down. Benign there only because the swallowed claim duplicated a real
      one; in the reverse count check a phantom source claim would
      additionally legitimize a fabricated case claim.
    - `package_corpus.rs:322` contains the fixture line
      `"./*": "./src/*.js"`, whose `/*` is inside a raw string. Blanking from
      there to the next `*/` (or, absent one, to end-of-file) blanked 13,084
      characters, and `_find_mod_declarations` returned `[]` for a file
      declaring five `#[path]` submodules -- see its own FIXED note.

    Both are measured, not hypothetical, and both are pinned:
    `test_double_slash_inside_a_string_is_not_a_comment` and
    `test_block_comment_open_inside_a_string_does_not_swallow_a_path_mod`.
    """
    out: list[str] = []
    i = 0
    n = len(source)
    while i < n:
        c = source[i]
        if c == '/' and i + 1 < n and source[i + 1] == '/':
            end = source.find('\n', i)
            if end == -1:
                end = n
            out.append(' ' * (end - i))
            i = end
            continue
        if c == '/' and i + 1 < n and source[i + 1] == '*':
            # RUST BLOCK COMMENTS NEST, and a naive `find('*/')` stops at the
            # INNER closer -- leaving everything between it and the true outer
            # close unmasked as live code. Reproduced end to end before this
            # fix: an `assert!(json[..].contains(..))` sitting between an inner
            # `*/` and its outer one was read as a real assertion and permitted
            # a `json_count` claim (`rc=0, AUDIT OK`, where it must refuse).
            #
            # Pre-existing rather than introduced by the reuse in
            # `json_leaf_contains_sites`: this branch has been naive since the
            # function was written, and that call site only made the
            # consequence reachable in a new direction. Dormant in this corpus
            # -- there is no genuine block comment in `crates/kali_cli/tests`
            # at all -- and dormancy is precisely why it is fixed on sight:
            # ruling 14's corpus differential cannot see a permission nobody
            # has exploited yet, so a green sweep is not evidence about it.
            depth, j = 1, i + 2
            while j < n and depth:
                if source.startswith('/*', j):
                    depth += 1
                    j += 2
                elif source.startswith('*/', j):
                    depth -= 1
                    j += 2
                else:
                    j += 1
            end = j if not depth else n
            segment = source[i:end]
            out.append(''.join(ch if ch == '\n' else ' ' for ch in segment))
            i = end
            continue
        if c in '"rbc\'':
            # `b`/`c` join the dispatch set for door 7: without them the scan
            # never OFFERS a `br#"..."#` open to `_skip_string`, so fixing that
            # function alone would have left this caller reading a `/*` inside a
            # byte raw string as a comment and blanking live code after it.
            # Measured before the fix, on a byte-raw fixture carrying `"./*"`:
            # the masker blanked from that `/*` to end of line. A bare `b"`/`c"`
            # still returns `None` here and falls through to the plain-string
            # branch on the next character, unchanged.
            end = _skip_string(source, i)
            if end is not None:
                out.append(source[i:end])
                i = end
                continue
        out.append(c)
        i += 1
    return ''.join(out)


def _mask_strings(source: str) -> str:
    """`source` (already comment-masked) with every string/char literal
    replaced by spaces (newlines preserved). Reuses `_skip_string` (below),
    the same masking technique `_blank_raw_strings` already uses for
    `JSON_KEY`. Used ahead of `PLAIN_MOD` matching only -- `PATH_MOD` needs
    its own `#[path = "..."]` string intact, so it runs on comment-masked-
    only text (see `_find_mod_declarations`)."""
    out: list[str] = []
    i = 0
    n = len(source)
    while i < n:
        c = source[i]
        # DOOR 8, THE THIRD PLACE THAT SHARED THE GUARD. Fix round 5 taught
        # `_RAW_STRING` and `_skip_string` about `br#"..."#` / `cr#"..."#` and
        # added `b`/`c` to `_mask_comments_outside_strings`'s DISPATCH set --
        # but not to this one, so `_skip_string` was never offered the opening
        # `b` and a byte raw string was not masked at all. Measured before the
        # fix:
        #
        #   _find_mod_declarations('br#"quote:" mod evil_phantom; end"#')
        #   -> [(None, 'evil_phantom')]        # a phantom submodule, minted
        #                                      # from the interior of a literal
        #   ... the same text as r#"..."#      -> []
        #
        # Direction: this arm feeds `_find_mod_declarations`, which creates a
        # DEMAND (the audit then resolves a submodule that does not exist and
        # exits 2), so the door fails LOUD -- which is why the controller parked
        # it rather than closing it with the permission-granting doors. It is
        # closed here because the condition attached to closing it is met and
        # measured: no verdict moves anywhere in the corpus (this batch's report
        # §17 carries the differential), and this batch changed no gate
        # semantics, so the differential is readable.
        if c in '"rbc' or c == "'":
            skip_end = _skip_string(source, i)
            if skip_end is not None:
                segment = source[i:skip_end]
                out.append(''.join(ch if ch == '\n' else ' ' for ch in segment))
                i = skip_end
                continue
        out.append(c)
        i += 1
    return ''.join(out)


def _find_mod_declarations(source: str) -> list[tuple[str | None, str]]:
    """Every submodule declaration in `source`, in order: `(explicit_path,
    mod_name)` where `explicit_path` is the `#[path = "..."]` string (or
    `None` for a plain `mod`/`pub mod`).

    FIXED (Task 18 pilot review round 3): both patterns used to run against
    raw source text, so a comment or string that merely MENTIONS a
    `mod x;`-shaped substring -- documentation prose describing the module
    structure (this corpus's own files do this constantly, e.g.
    `inprocess/cdp_driver.rs`'s `//!` header, "...resolve its unqualified
    `mod cdp_driver;`..."), or a fixture string, or a commented-out
    declaration -- was read as a real one, live in this corpus (harmless
    there only because the phantom name happened to already be visited).

    `PATH_MOD` is matched against a COMMENT-masked (not string-masked) copy
    of `source`: a `#[path = "..."]` attribute's own string argument must
    stay intact for its capture group to read the real path, so blanking
    strings before this pass would blank that argument along with every
    other string -- an earlier version of this fix did exactly that and
    broke every `#[path]` resolution in the corpus (caught immediately by
    the existing regression suite, not shipped). `PATH_MOD`'s full matched
    spans (including that intact string) are then blanked out of a second,
    NOW string-masked copy, and `PLAIN_MOD` runs against that -- so a
    `#[path = "a.rs"] mod b;` is never ALSO picked up as a plain `mod b;`
    (which would resolve the wrong file, `b.rs`/`b/mod.rs`, instead of the
    explicit `a.rs`), and a `mod x;` inside any OTHER string or comment is
    invisible to the plain-mod pass.

    FIXED (Task 18 audit-count fix round, found by review): that comment mask
    is `_mask_comments_outside_strings`, not `_mask_comments`, because the
    latter is not string-aware -- a `/*` or `//` INSIDE a string literal is
    read as a comment open. Live in this corpus, and not harmlessly:
    `package_corpus.rs:322` contains the fixture line `"./*": "./src/*.js"`,
    whose `/*` made `_mask_comments` blank **13,084 characters** through
    end-of-file, so this function returned `[]` for a file that declares FIVE
    `#[path]` submodules at `:754-767`. The danger direction here is a MISSED
    submodule, not a wrong one (a runaway blanks text, it does not mint
    declarations -- swept over all 228 live and 99 restored sources against a
    string-aware ground truth, `package_corpus.rs` is the only difference and
    there are no spurious extras): a parent with at least one `#[test]` fn
    plus a silently dropped submodule audits `AUDIT OK` over claims nobody
    examined. `package_corpus.rs` itself is fail-closed today -- it declares
    zero top-level `#[test]` fns, so auditing it hits the "0 #[test] fns"
    refusal rather than a silent OK -- but that is a property of one file, not
    of the resolver."""
    comments_masked = _mask_comments_outside_strings(source)
    out: list[tuple[str | None, str]] = []
    path_mod_matches = list(PATH_MOD.finditer(comments_masked))
    for match in path_mod_matches:
        out.append((match.group(1), match.group(2)))

    fully_masked = _mask_strings(comments_masked)
    working = fully_masked
    for match in path_mod_matches:
        start, end = match.span()
        working = working[:start] + (" " * (end - start)) + working[end:]
    for match in PLAIN_MOD.finditer(working):
        out.append((None, match.group(1)))
    return out


def _resolve_one_mod(
    including_path: Path, plain_base: Path, explicit_path: str | None, mod_name: str
) -> tuple[Path, Path]:
    """Resolve one `mod_name` declaration found in `including_path`.
    Returns `(resolved_file, base_for_its_own_plain_mod_children)` -- see
    the module-level comment above `PATH_MOD` for the directory-style vs
    leaf-style distinction the second element encodes.

    `explicit_path` (a `#[path]` string) always resolves relative to
    `including_path.parent` -- never `plain_base` -- since `#[path]` is not
    subject to the nesting convention `plain_base` exists to track. A
    resolved `#[path]` target is always directory-style for ITS OWN
    children (its own directory becomes their base).

    A plain `mod_name` (no `#[path]`) resolves relative to `plain_base`:
    `mod_name.rs` is tried first (leaf-style: a further nested plain mod
    inside it would need `mod_name/`, so its own children's base is
    `flat.parent / flat.stem`), then `mod_name/mod.rs` (directory-style:
    already its own directory). If neither exists, the `.rs` candidate is
    returned anyway (with a leaf-style child base, moot since resolution
    stops at the caller's existence check) so the caller produces one
    clear, single-path error message rather than this function raising.
    """
    if explicit_path is not None:
        resolved = including_path.parent / explicit_path
        return resolved, resolved.parent
    flat = plain_base / f"{mod_name}.rs"
    if flat.is_file():
        return flat, flat.parent / flat.stem
    nested = plain_base / mod_name / "mod.rs"
    if nested.is_file():
        return nested, nested.parent
    return flat, flat.parent / flat.stem


def resolve_path_mods(old_path: Path, source: str) -> list[Path]:
    """Every submodule file reachable from `old_path`/`source` by following
    `#[path = "..."] mod ...;` and plain `mod ...;`/`pub mod ...;`
    declarations, RECURSIVELY (a resolved submodule's own mod declarations
    are followed too, with the correct per-file base directory -- see
    `_resolve_one_mod`), in breadth-first discovery order.

    The visited-set holds `Path.resolve()`d (truly canonical, `..`-collapsed
    and symlink-resolved) paths, not the lexically-assembled ones `resolved`
    returns -- so a cycle spelled through a `..`-climbing `#[path]` that
    never lexically repeats (e.g. `sub/mod.rs` containing
    `#[path = "../sub/mod.rs"] mod sub;`, which re-derives a
    longer-and-longer but never-identical string on every hop without
    `.resolve()`) is still caught on the first repeat, not after `is_file()`
    starts failing on an over-long path. `Path.resolve(strict=False)` (the
    default) works on non-existent paths too, so this applies uniformly
    whether or not the target exists."""
    resolved: list[Path] = []
    visited: set[Path] = {old_path.resolve()}
    # The top-level file is always directory-style for module-resolution
    # purposes (a crate/binary root, not a leaf module of some parent).
    queue: list[tuple[Path, str, Path]] = [(old_path, source, old_path.parent)]
    while queue:
        current_path, current_source, plain_base = queue.pop(0)
        for explicit_path, mod_name in _find_mod_declarations(current_source):
            child, child_plain_base = _resolve_one_mod(
                current_path, plain_base, explicit_path, mod_name
            )
            child_resolved = child.resolve()
            if child_resolved in visited:
                continue
            visited.add(child_resolved)
            resolved.append(child)
            if child.is_file():
                queue.append((child, child.read_text(), child_plain_base))
            # A missing child is left for `main`'s existence check (which
            # reports it with the same "does not exist" message either
            # shape produces) -- not re-checked or raised here, so every
            # missing submodule in a file is reported, not just the first.
    return resolved

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
# DOOR 7: THE PREFIX SET IS `r`, `br`, `cr` -- ASKED OF rustc, NOT REMEMBERED.
# The lookbehind used to sit directly on `r`, so for a BYTE raw string
# (`br"..."`, `br#"..."#`) the preceding `b` counted as an identifier character,
# the guard fired, and the literal was never recognised as a raw string at all.
# A `.contains(...)` inside one was then read as live code -- door 5's exact
# class through the byte spelling, demonstrated on a running gate.
#
# The lookbehind now sits before the whole prefix, which keeps the guard it was
# there for: in `xbr"`, the attempt at `b` sees `x` and the attempt at `r` sees
# `b`, so both fail; a word ending in `r` (`"...operator"`) still cannot open a
# raw string.
#
# `rb"` IS NOT A RUST LITERAL and is deliberately absent. rustc 1.97.1:
#
#     $ printf 'fn main() { let _ = rb"x"; }\n' > p.rs && rustc ... p.rs
#     error: prefix `rb` is unknown
#
# while `r"x"`, `br"x"`, `br#"x"#`, `br##"x"##`, `cr"x"`, `c"x"` and `b"x"` all
# compile. `c"..."` and `b"..."` are NOT raw -- they are escaped literals and
# the plain `"` path already handles them; only the `r`-carrying prefixes belong
# here. U15's standing prohibition names `br"` and `cr"` for a reason.
_RAW_STRING = re.compile(r'(?<![A-Za-z0-9_])(?:br|cr|r)(#*)"(?:.*?)"\1', re.DOTALL)


_LET_BINDING = re.compile(
    r"\blet\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*(?::[^=;]*)?=\s*([^;]+);", re.S)
_LEADING_INDEX = re.compile(r'\A\s*\[\s*(?:"([^"]*)"|\'([^\']*)\'|(\d+))\s*\]')
_HEAD_IDENT = re.compile(r"\A\s*&?\s*([A-Za-z_][A-Za-z0-9_]*)")
# ESCAPE HANDLING, CORRECTED IN TASK 19 BATCH 5. This was
# `(\"(?:[^\"\\\\]|\\\\.)*\")`, whose alternation spells FOUR backslashes and
# therefore required TWO literal backslashes before the escaped character. A
# Rust `.contains("7\\n7\\n")` -- one backslash, the overwhelmingly common
# spelling in this corpus -- matched nothing at all, so every json-leaf
# `.contains` whose needle carries an escape was invisible to
# `json_leaf_contains_sites`.
#
# WHICH DIRECTION THAT FAILED IN, because it decides what evidence is needed.
# This regex feeds ONE arm, the acceptance at the count-claim check, which
# GRANTS permission for a `json_count { at_least = 1 }` pinned at the leaf the
# source's `.contains` was taken on (ruling 3's amended clause 4). Missing a
# site therefore made the gate REFUSE a compliant claim -- a false red, the safe
# direction -- and the corrected pattern matches a strict superset of what the
# old one did, so no already-passing pair can be flipped by it. That is why a
# corpus differential is necessary and NOT sufficient here: it cannot see a
# permission nobody has exploited. The evidence for the grant is the refusal
# suite enumerated against the language in `audit-case-migration_test.py` --
# every one of the five doors this function's docstring names, plus the two
# escape-specific ones this correction opens the door for.
_CONTAINS_CALL = re.compile(r'\.\s*contains\s*\(\s*("(?:[^"\\]|\\.)*")\s*\)')


def _json_path_of(expr: str, env: dict) -> str | None:
    """The dotted json path an expression addresses, or None if it is not a
    json leaf. `json` is the root; a `let`-bound name inherits the path it was
    bound to. Only the index chain immediately after the head identifier is
    read -- everything after it (`.as_str()`, `.expect(...)`) is a coercion,
    not a path segment."""
    m = _HEAD_IDENT.match(expr)
    if not m:
        return None
    head = m.group(1)
    if head == "json":
        path: list[str] = []
    elif head in env:
        path = env[head].split(".") if env[head] else []
    else:
        return None
    rest = expr[m.end():]
    while True:
        idx = _LEADING_INDEX.match(rest)
        if not idx:
            break
        path.append(next(g for g in idx.groups() if g is not None))
        rest = rest[idx.end():]
    return ".".join(path)


def _receiver_expression(text: str, dot: int) -> str:
    """The receiver chain ending at `text[dot]`, walked backwards with bracket
    balancing. Returns "" when the walk finds no expression."""
    close = {")": "(", "]": "[", "}": "{"}
    depth: list[str] = []
    i = dot - 1
    while i >= 0:
        c = text[i]
        if c in close:
            depth.append(close[c])
            i -= 1
            continue
        if c in "([{":
            if depth and depth[-1] == c:
                depth.pop()
                i -= 1
                continue
            break
        if depth:
            i -= 1
            continue
        if c.isalnum() or c in "_.\"'":
            i -= 1
            continue
        if c.isspace():
            # A postfix chain in this corpus is routinely wrapped across lines
            # (`json["stdout"]\n    .as_str()\n    .expect(..)\n    .contains(..)`),
            # so whitespace INSIDE the chain has to be walked over -- but
            # whitespace that precedes the chain's head must stop the walk, or
            # the receiver swallows `assert!(` and everything before it. Skip
            # the run, then decide on what it separates: a character that can
            # END an expression fragment continues the chain, anything else
            # (`(`, `!`, `,`, `=`) is the boundary.
            j = i
            while j >= 0 and text[j].isspace():
                j -= 1
            if j >= 0 and (text[j].isalnum() or text[j] in "_.\"')]"):
                i = j
                continue
            break
        break
    return text[i + 1:dot].strip()


def json_leaf_contains_sites(source: str) -> set:
    """`{(dotted json path, needle)}` for every `.contains("lit")` in `source`
    whose RECEIVER is the json leaf at that path.

    THE RECEIVER REQUIREMENT IS THE WHOLE SPECIFICATION, and it is what the
    first version of the count-claim acceptance lacked. Ruling 3's amended
    clause 4 reads "plain `.contains(x)` against a JSON STRING LEAF", and its
    entire content is the correspondence between the source's json leaf and the
    path the case file pins. Accepting any `.contains` anywhere in the file
    instead let FIVE things through, each demonstrated on a running gate rather
    than argued, and each pinned as a refusal test below:

      1. a `.contains` inside a `//` line comment;
      2. a `.contains(` inside a JS fixture raw string;
      3. a `.contains` on RAW STDOUT, pinned as a `json_count` at a json path;
      4. a `.contains` on ONE json leaf, pinned at ANOTHER path;
      5. a `.contains` inside a `/* ... */` BLOCK comment.

    3 and 4 are refused by construction, because the path has to match. 1, 2 and
    5 are refused by what this line reads: raw strings blanked, then comments
    masked -- and the masker is `_mask_comments_outside_strings`, the one this
    file already had, which is string-aware and handles BOTH comment forms.

    THE FIRST FIX HERE WROTE A NARROWER MASKER BESIDE IT, handling only `//`,
    and its docstring said the job was done. That left door 5 open in the very
    round that existed to close this class. Door 5 is dormant -- there is no
    genuine block comment in `crates/kali_cli/tests/*.rs` today -- and dormancy
    is exactly why it had to be closed on sight: ruling 14's corpus differential
    cannot see a permission nobody has exploited yet, so a green sweep is not
    evidence about this arm.

    ORDER, STATED AS WHAT IS ACTUALLY TRUE. An earlier version of this
    docstring said the order was load-bearing, and it is not: the masker is
    INDEPENDENTLY string-aware -- it recognises `r#"..."#` through the same
    `_skip_string` primitive `_blank_raw_strings` uses -- so a `/*` inside a
    fixture body (`"./*": "./src/*.js"` in this corpus) is safe whichever pass
    runs first. Measured: the two orders produce byte-identical text on every
    `.rs` under `crates/kali_cli/tests`, and that equality is asserted every
    gate run by `test_the_two_masking_passes_commute_on_this_corpus` rather
    than claimed here.

    They differ on exactly one shape, which no source in this tree contains: a
    raw string carrying a comment closer inside a block comment,
    `/* r#"*/"# ... */`. Rust's lexer does not respect strings inside block
    comments, so the comment really ends at that first `*/` and the INVERTED
    order is the one that matches the language; the order used here masks
    further and refuses, which is the conservative direction for an arm that
    grants permission rather than demanding coverage. Recorded because a reader
    who finds that input should know the divergence is known, not accidental.
    """
    text = _mask_comments_outside_strings(_blank_raw_strings(source))
    env: dict[str, str] = {}
    for m in _LET_BINDING.finditer(text):
        path = _json_path_of(m.group(2), env)
        if path is not None:
            env[m.group(1)] = path
    out = set()
    for m in _CONTAINS_CALL.finditer(text):
        receiver = _receiver_expression(text, m.start())
        if not receiver:
            continue
        path = _json_path_of(receiver, env)
        if path:
            out.add((path, unquote(m.group(1))))
    return out


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

    "Raw string" here means all three prefixes rustc accepts -- `r`, `br`, `cr`
    -- not just the bare `r` this function keyed on until door 7. `b"..."` and
    `c"..."` are escaped literals and are NOT raw, so they are deliberately
    outside this masking and are covered by the paragraph below.

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


def _char_literal_end(text: str, pos: int) -> int | None:
    """If `text[pos] == "'"` opens a genuine Rust CHAR literal (`'('`,
    `','`, `'\\n'`, `'\\''`, `'\\x41'`, `'\\u{2764}'`, ...), return the index
    one past its closing `'`; otherwise `None`.

    FIXED (Important, found during Task 17 review): `_split_top_level_args`/
    `_find_calls` previously treated `(`/`)`/`[`/`]`/`{`/`}` as bare bracket
    characters everywhere, including inside a char literal -- so
    `assert_eq!(s.replace('(', "x"), "expected\\n")` read the `(` inside
    `'('` as a real paren-depth increment with no matching decrement (the
    literal's own closing `'` isn't a bracket, so depth is left one too
    high), corrupting the argument split for the rest of that call and
    silently losing `"expected\\n"` -- worse than the bug this replaced,
    which at least matched (wrongly); this one goes quietly missing, which
    reads as `AUDIT OK`. A char literal containing `)`/`]`/`}` instead can
    additionally make `_find_calls`'s depth cross zero on the WRONG
    character, ending the call's argument-list scan early (or, in the
    unclosed-bracket direction, never returning to zero and swallowing the
    rest of the file as one `arg_text`, which can mint phantom claims from
    unrelated later code).

    The ambiguity this guards against is Rust LIFETIMES (`'a`, `'static`,
    `&'a str`), which also start with a bare `'` followed by an identifier
    character and are never closed by a second `'` at all. The guard is
    exactly that: a candidate is only accepted as a char literal if a
    plausible closing `'` is found within the short, escape-shaped window a
    real char literal's body can occupy (at most `'\\u{10FFFF}'`, 10
    characters including both quotes) -- a lifetime's identifier is not
    followed by a `'` at all, so it never matches and is correctly left as
    ordinary bracket-free text (which is what it is: identifiers and `&`
    carry no bracket-depth meaning either)."""
    n = len(text)
    if pos + 1 >= n:
        return None
    if text[pos + 1] == '\\':
        # Escape body: \xHH, \u{1-6 hex}, or a single-character escape
        # (\n \t \r \\ \' \" \0), each followed immediately by the closing '.
        if pos + 2 >= n:
            return None
        e = text[pos + 2]
        if e == 'x' and pos + 5 <= n and text[pos + 5:pos + 6] == "'":
            return pos + 6
        if e == 'u' and pos + 3 < n and text[pos + 3] == '{':
            close_brace = text.find('}', pos + 4)
            if close_brace != -1 and close_brace + 1 < n and text[close_brace + 1] == "'":
                return close_brace + 2
            return None
        if e in ('n', 't', 'r', '\\', "'", '"', '0') and pos + 3 < n and text[pos + 3] == "'":
            return pos + 4
        return None
    # Plain single character (never a bare `'`, which would be the closing
    # quote itself -- Rust requires it escaped as `\'` inside a char literal,
    # already handled above).
    if text[pos + 1] != "'" and pos + 2 < n and text[pos + 2] == "'":
        return pos + 3
    return None


def _skip_string(text: str, pos: int) -> int | None:
    """If `text[pos]` opens a string literal (plain `"..."` or raw
    `r#*"..."#*`) or a char literal (`'x'`, see `_char_literal_end`), return
    the index one past its closing delimiter; otherwise `None`. A raw-string
    open is only recognized when `pos` is a genuine token start (the
    character before it, if any, is not an identifier character) -- the same
    guard `_RAW_STRING` above needs and for the identical reason (an
    ordinary word ending in `r`, e.g. `"...operator")`, must not be misread
    as the start of a raw string)."""
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
    if c in 'brc' and (pos == 0 or not (text[pos - 1].isalnum() or text[pos - 1] == '_')):
        # DOOR 7, the second half. This guard had the identical defect as
        # `_RAW_STRING`'s and for the identical reason -- it keyed on `r` alone,
        # so `br#"..."#` was not recognised as a string here either, and a `//`
        # or `/*` inside a byte raw string would have been masked as a comment,
        # swallowing live code after it. The two are fixed together because they
        # are one rule spelled twice.
        #
        # `b`/`c` are accepted only as a prefix to `r`; a bare `b"..."` or
        # `c"..."` is an ESCAPED literal, falls through to `None` here, and is
        # handled by the plain `"` branch on the next character exactly as
        # before. That is what keeps the files using `b"..."` unmoved.
        #
        # NO INTEGER HERE, DELIBERATELY (ruling 15's answer 3, and ruling 16's
        # general form). This comment used to say "the 14 files" and the
        # regression test's docstring said "the 24 files" -- two different
        # numbers for one population, both in the commit that introduced them.
        # The 24 came from an unanchored `grep -rl 'b"'`, which also matches
        # `.arg("--lib")`; the word-boundary count is what a byte-string opener
        # actually needs:
        #
        #   grep -rlE '(^|[^A-Za-z0-9_])b"' crates/kali_cli/tests --include=*.rs | wc -l
        #
        # and the answer is a live corpus count that Task 20's source deletions
        # will change again, so writing the corrected integer would only reset
        # the clock. The CLASS is what this comment is about, and the property
        # is gated by `test_raw_string_prefixes_match_what_rustc_accepts`.
        k = pos + 1
        if c != 'r':
            if k >= n or text[k] != 'r':
                return None
            k += 1
        hashes = 0
        while k < n and text[k] == '#':
            hashes += 1
            k += 1
        if k < n and text[k] == '"':
            close = '"' + ('#' * hashes)
            end = text.find(close, k + 1)
            return (end + len(close)) if end != -1 else n
        return None
    if c == "'":
        return _char_literal_end(text, pos)
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
        # `b`/`c` are in the dispatch set for the same reason they are in
        # `_blank_raw_strings`'s and `_mask_comments_outside_strings`'s: without
        # them the scan never OFFERS a `br#"..."#` open to `_skip_string`, whose
        # own guard then rejects the inner `r` because the preceding `b` is an
        # identifier character -- so the raw string's interior is split on its
        # own commas. Measured before the fix, on `br#"say " and , here"#, x`:
        # `['br#"say " and', 'here"#, x']` instead of two arguments. Task 19
        # batch 4 enumerated this class repo-wide rather than finding a seventh
        # instance the way the first six were found, one at a time.
        if c in '"rbc' or c == "'":
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
            # Same dispatch set, same reason, same class: without `b`/`c` a
            # `br#"..."#` argument's interior parens are counted as the call's
            # own. Measured on `f(br#"a " b ) c"#, y)`: the call text came back
            # truncated at the raw string's interior `)`.
            if c in '"rbc' or c == "'":
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
            if re.fullmatch(_STR_LITERAL, a, _STR_LITERAL_FLAGS):
                tokens.append(a)
    return tokens


# A `.matches("literal")` ... `.count()` occurrence-count expression. The
# whitespace slots are not decoration: this corpus wraps the chain across
# lines whenever the receiver is long (`browser_math_log2_log10.rs`'s json
# branch spells it as `json["stdout"]\n.as_str()\n.expect(...)\n
# .matches("3\n")\n.count()\n>= 2`), and an arm that only matched the
# single-line spelling would read the raw-stdout branch of every migrated
# helper and silently miss the json branch of the same helper -- half a
# claim kind, in the direction that reports OK.
_COUNT_MATCHES = re.compile(
    rf'\.\s*matches\(\s*(?:&)?({_STR_LITERAL})\s*\)\s*\.\s*count\(\s*\)',
    _STR_LITERAL_FLAGS,
)
# The comparison immediately following a `.count()` inside an `assert!`:
# `>= K` (CountBound::AtLeast) or `== K` (CountBound::Exact). Deliberately
# closed at the two comparisons `model.rs`'s `CountBound` admits -- a `<`/
# `>`/`!=` count assertion has no representable case-file form, so pairing
# one with a bound here would invent a correspondence that cannot exist.
# A `.count()` whose comparison this does not recognize still yields its
# NEEDLE as a claim; only the bound goes unaudited, and `main` says so out
# loud rather than passing it silently.
_COUNT_BOUND_TAIL = re.compile(r'\s*(>=|==)\s*([0-9]+)')
_INTEGER_LITERAL = re.compile(r'[0-9]+')


def count_claim_sites(source: str) -> list[tuple[str, tuple[str, int] | None]]:
    """Every occurrence-count claim asserted in `source`, as
    `(raw_literal_token, bound)` where `bound` is `("at_least"|"exact", K)`
    or `None` when the comparison was not one of the two representable ones.

    Only `.matches("lit").count()` occurrences inside an `assert!`/
    `assert_eq!` argument count: outside an assertion the same expression is
    arithmetic, not a claim (see the module docstring). The scan reuses
    `_find_calls`/`_split_top_level_args` -- the same balanced-paren,
    string-aware machinery `_assert_eq_literal_tokens` uses -- rather than a
    regex over raw text, so a `)` inside a fixture string cannot end an
    assertion's argument list early.

    Only the condition argument of `assert!` (index 0) and the two compared
    arguments of `assert_eq!` (indices 0 and 1) are read; a trailing panic-
    message format string is never inspected, matching every other arm's
    scope.

    The scan runs over `_mask_comments_outside_strings(source)`, unlike the other
    claim arms.
    That is deliberate and it is not the (deliberately out-of-scope) fix for
    this script's known phantom-claims-from-`//!`-prose defect -- it is a
    refusal to INHERIT it here, for a reason specific to this arm: count
    claims are the one kind checked in the reverse direction too
    (`count_claim_correspondence`), so a §5.11 retention header quoting
    `assert!(stdout.matches("0\\n").count() >= 3)` as prose would not merely
    manufacture a phantom claim against the case files -- it would ALSO
    manufacture a source assertion for a fabricated case claim to correspond
    to, i.e. it would weaken the fabrication check with text that no compiler
    ever saw. Every file the count keys exist to rescue carries exactly such
    a header (`browser_math_asinh_acosh_atanh_identities.rs`'s quotes
    `stdout.matches(<needle>).count() >= 3` verbatim), so this is live, not
    theoretical. The other arms are unchanged and still read `//!` prose;
    that defect is recorded elsewhere and is not touched here.
    """
    source = _mask_comments_outside_strings(source)
    out: list[tuple[str, tuple[str, int] | None]] = []
    for macro, arity in (("assert!", 1), ("assert_eq!", 2)):
        for arg_text in _find_calls(source, macro):
            args = _split_top_level_args(arg_text)
            for index, argument in enumerate(args[:arity]):
                for match in _COUNT_MATCHES.finditer(argument):
                    bound: tuple[str, int] | None = None
                    tail = _COUNT_BOUND_TAIL.match(argument[match.end():])
                    if tail:
                        bound = (
                            "at_least" if tail.group(1) == ">=" else "exact",
                            int(tail.group(2)),
                        )
                    elif macro == "assert_eq!" and len(args) >= 2:
                        # `assert_eq!(x.matches(L).count(), K)` -- the bound
                        # is the OTHER compared argument, on either side of
                        # the comma (`assert_eq!(K, x.matches(L).count())`
                        # is the same claim written backwards).
                        other = args[1 - index].strip()
                        if _INTEGER_LITERAL.fullmatch(other):
                            bound = ("exact", int(other))
                    out.append((match.group(1), bound))
    return out


class _CountNeedles:
    """Duck-types `re.Pattern`'s `.findall(source) -> list[str]` so
    `LITERAL_KINDS` can hold the count-claim scanner alongside the real
    compiled regexes, exactly as `_AssertEqLiterals` does."""

    def findall(self, source: str) -> list[str]:
        return [token for token, _bound in count_claim_sites(source)]


COUNT_NEEDLES = _CountNeedles()


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
    "count needles": [COUNT_NEEDLES],
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
    # Same reasoning, same two values, for count needles: `runtime_smoke`'s
    # `stdout.matches("0").count() >= 3` needles a bare "0", whose presence
    # in the joined new text discriminates nothing. Excluding it costs
    # nothing real -- a count claim's substance (its bound, and whether its
    # needle corresponds to a source claim at all) is checked structurally
    # by `count_claim_correspondence`, which BORING does not touch.
    "count needles": {"0", "1"},
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

# The Rust string-continuation escape: a backslash immediately followed by a
# REAL embedded newline (two literal source bytes: `\` then an actual `\n`
# character -- not the two-character escape `\n`, backslash-then-letter-n,
# which `unquote()`'s plain `.replace("\\n", "\n")` already handles). Rust
# drops the backslash, the newline, and every whitespace character
# immediately following it up to the next non-whitespace byte. FIXED (Minor
# 3 follow-on, found while fixing `CONST`'s missing `re.DOTALL`, same
# session): applying `re.DOTALL` alone made `CONST`/`CONTAINS` *match* a
# continuation-wrapped literal like `switch_runtime.rs`'s `const S`/`const
# SS`, but `unquote()` had no rule for this escape at all, so the "canonical"
# text it produced still contained the literal backslash-plus-newline
# sequence the source's own compiler would have dropped -- a real, working
# TOML value could never equal that wrong canonical string, so the DOTALL fix
# alone would have taken `switch_runtime.rs` from a false "rule constants: 0"
# straight to a false "AUDIT FAILED" on its own (now-hoisted) `[constants]`
# entries. The `(?<!\\)` guard is the same shape as `_UNICODE_ESCAPE`'s: it
# must run before the final `\\\\` -> `\\` collapse (see that ordering note
# below) so a genuine escaped-backslash-then-real-newline (`"\\\` followed by
# a physical newline, meaning a literal trailing backslash character, not a
# continuation) is not misread as a continuation and does not have its
# newline silently eaten. Not observed in this corpus at the time of this
# fix, but the same cheap-to-guard-against posture as `_UNICODE_ESCAPE`.
_LINE_CONTINUATION = re.compile(r'(?<!\\)\\\n[ \t\r\n]*')


def unquote(raw: str) -> str:
    """Turn a Rust string literal token into its fully-unescaped text.

    `_LINE_CONTINUATION` runs first (right after stripping the outer quotes),
    strictly before every other substitution in this function -- see its own
    comment for why it must precede the final `\\\\` -> `\\` collapse.

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
    trigger it), and re-run before and after against the `.rs`/`.toml` pairs
    that were migrated then -- the case files at that fix's own base, plus
    `soundness/textcodec.toml`, which the same review added:

        git ls-tree -r --name-only ec895f8149 \\
            -- crates/kali_cli/tests/cases/{string,array,math,object,soundness}

    THE COUNT THAT USED TO SIT IN THAT SENTENCE IS DELETED, NOT CORRECTED
    (batch 8-inst-2 fix round 1). It read "all 50 then-migrated pairs ...
    50/50 both times". It was exact when written and describes a population
    that has grown in every batch since, because it counts files this script
    does not contain and nothing fails when it stops being true -- the same
    disposition, and the same ruling 15/16 reasoning, applied to its twin in
    `soundness/textcodec.toml:101-103`. Deleting one figure and leaving its
    identical twin two files away is the inconsistency the rule exists to
    stop. The command above is pinned to an immutable ref and cannot go
    stale; THIS SCRIPT is the live side, so anyone re-deriving the result
    must say which revision of it they ran.
    """
    raw = raw.strip()
    # I1 (batch 4 fix round 1): `raw.startswith("r")` decided raw-ness, so
    # `unquote('br#"a"#')` returned `'r#"a"'` -- the `b` was read as the opening
    # quote character. This was DORMANT only because `_STR_LITERAL` above was
    # raw-blind and therefore never handed this function a `b`/`c` token. The two
    # are fixed in one commit for that reason: closing the recogniser alone would
    # have taken a latent mangling function live.
    m = _RAW_LITERAL_OPEN.match(raw)
    if m:
        hashes = len(m.group(1))
        return raw[m.end() : len(raw) - hashes - 1]
    body = raw[1:-1]
    body = _LINE_CONTINUATION.sub('', body)
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
    if _RAW_LITERAL_OPEN.match(raw):        # I1: was `raw.startswith("r")`
        return unquote(raw)
    return raw[1:-1]


def literal_variants(token: str) -> frozenset[str]:
    """Both spellings a Rust string literal's contents might take in a TOML
    case file: as written (escapes intact) and fully unescaped. See the
    module docstring for why both must be checked."""
    return frozenset({raw_body(token), unquote(token)})


def disjunctive_contains_groups(source: str) -> list[dict]:
    """Groups of `.contains(...)` literals that the source asserts DISJUNCTIVELY.

    Returns `[{"literals": [canonical, ...], "sites": frozenset[offset]}, ...]`.
    `sites` are the absolute offsets, into `source`, of the `.contains(` matches
    that make up the group, and they are what makes suppression SITE-scoped
    rather than literal-scoped -- see `contains_sites` for why that distinction
    is the difference between this arm and a false `AUDIT OK`.

    Rule 11 is explicit that an OR-shaped source assertion is resolved against
    the real binary and the branch that actually occurs is pinned -- "a verified
    strengthening (every run satisfying the new assertion satisfies the old)".
    This script's claim model is conjunctive: every extracted literal must
    appear somewhere in the case files. Those two are in direct conflict for the
    one OR shape where the disjuncts are DIFFERENT literals rather than the same
    code on two streams:

        assert!(
            message.contains("array callback-produced iterables")
                || message.contains("literal array"),
            "unexpected error message: {message}"
        );

    The source does not claim both texts are present -- it claims at least one
    is -- so requiring both is the tool asserting something the source never
    did, and it fails a migration that is strictly STRONGER than its
    predecessor. Measured when this was added, by running this very function over
    every `browser_*.rs` with its `#[path]` submodules resolved: EIGHT targets
    form a group -- `math_atan2_global_this_root`,
    `math_unsupported_member_calls_harness_jsx_tsx`,
    `non_literal_dynamic_import_harness_jsx_tsx`, `non_literal_iterator_sources`,
    `object_keys_entries_spread_bundle`, `object_keys_entries_spread_harness`,
    `object_values_spread_harness`, `wasm_threads_browser_surface`. Of the 79
    already-shipped stem-matched pairs, ZERO change verdict and zero even print a
    DISJUNCTION line -- checked by running the whole family against a copy of this
    script with the arm neutered and diffing every pair's output; in the two
    shipped pairs that do form a group no member is present, so nothing is
    suppressed. The same-literal-two-streams shape rule 11's evidence cites --
    `stderr.contains("E5506") || stdout.contains("E5506")` -- never forms a group
    at all: it has ONE distinct literal, and whichever stream is pinned carries it
    either way.

    So a group is required to have AT LEAST ONE member present, not all -- and
    an unpinned member is suppressed only if EVERY `.contains` site of that
    literal in the combined source lies inside a satisfied group (fix round 1,
    C1; `contains_sites`). The resolution is never silent: `main` prints a
    DISJUNCTION line naming the group, the member that satisfied it, which
    unpinned branches were suppressed and which were NOT and why, exactly as it
    prints UNAUDITED and NOT MIRRORED notes.

    FAILS CLOSED, deliberately. A group is formed only when the assertion is a
    pure top-level disjunction: the balanced-paren macro body is split at
    depth-zero `||`, and the group is abandoned (every literal reverting to an
    independent, conjunctive claim) if any disjunct contains a top-level `&&`,
    or if fewer than two disjuncts yield a literal. A mixed `a && (b || c)` is
    therefore audited exactly as it was before this function existed. Nothing
    here can turn a wholesale drop green: if NO member of a group appears in the
    case files, every member is still reported missing.
    """
    masked = _mask_strings(_mask_comments_outside_strings(source))
    groups: list[list[str]] = []
    for m in re.finditer(r"\bassert!\s*\(", masked):
        depth, i, n = 0, m.end() - 1, len(masked)
        while i < n:
            if masked[i] == "(":
                depth += 1
            elif masked[i] == ")":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        if i >= n:
            continue
        body_start, body_end = m.end(), i
        # Split the MASKED body at depth-zero `||`, then read the literals out
        # of the corresponding slice of the UNMASKED source.
        cuts, depth = [body_start], 0
        j = body_start
        while j < body_end:
            ch = masked[j]
            if ch in "([{":
                depth += 1
            elif ch in ")]}":
                depth -= 1
            elif depth == 0 and masked.startswith("||", j):
                cuts.append(j + 2)
                j += 2
                continue
            j += 1
        cuts.append(body_end + 1)
        if len(cuts) < 3:                      # no top-level `||`
            continue
        members: list[str] = []
        sites: set[int] = set()
        ok = True
        for a, b in zip(cuts, cuts[1:]):
            segment_masked = masked[a:b - 1]
            if _has_top_level(segment_masked, "&&"):
                ok = False
                break
            for hit in CONTAINS.finditer(source[a:b - 1]):
                members.append(unquote(hit.group(1)))
                # ABSOLUTE offset into `source`. This is what makes suppression
                # SITE-SCOPED rather than literal-scoped; see the `sites` note
                # in this function's docstring.
                sites.add(a + hit.start())
        if ok and len(set(members)) >= 2:
            groups.append({"literals": sorted(set(members)), "sites": frozenset(sites)})
    return groups


def contains_sites(source: str) -> dict[str, frozenset[int]]:
    """Every `.contains("lit")` site in `source`: canonical literal -> offsets.

    The other half of site-scoped suppression. A literal may be an unpinned
    disjunct at one site and an UNCONDITIONAL claim at another --
    `browser_wasm_threads_browser_surface.rs` does exactly that with
    `"runtime profile"` (`:31` inside the OR, `:81` on its own in the JSON
    branch) -- so suppressing by literal alone makes `AUDIT OK` mean "a claim
    the source asserts unconditionally is absent". Standing ruling R2 forbids
    precisely that. Offsets are produced by the same `CONTAINS` pattern
    `disjunctive_contains_groups` records its sites with, so the two sets are
    directly comparable.
    """
    out: dict[str, set[int]] = {}
    for hit in CONTAINS.finditer(source):
        out.setdefault(unquote(hit.group(1)), set()).add(hit.start())
    return {k: frozenset(v) for k, v in out.items()}


def _has_top_level(segment: str, operator: str) -> bool:
    depth = 0
    i = 0
    while i < len(segment):
        ch = segment[i]
        if ch in "([{":
            depth += 1
        elif ch in ")]}":
            depth -= 1
        elif depth == 0 and segment.startswith(operator, i):
            return True
        i += 1
    return False


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
# Occurrence-count claim keys. These are lists of TABLES, not lists of
# strings, which is exactly why naming them in `_STEP_LIST_KEYS` would be a
# no-op: that tuple's consumer filters `isinstance(v, str)` and would discard
# every table. They need their own reader (`_step_count_claims`), and they get
# one -- `Invariant8`'s key-sync test only proves a key is NAMED in one of
# these tuples, so `CountKeyExtraction` in the regression suite pins the
# extractor's OUTPUT for every key in all four tuples instead.
_STEP_COUNT_KEYS = ("stdout_count", "json_count")
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
    for _key, claim in _step_count_claims(step):
        for field in ("needle", "path"):
            value = claim.get(field)
            if isinstance(value, str):
                out.append(value)
    return out


def _step_count_claims(step: dict) -> list[tuple[str, dict]]:
    """Every `stdout_count`/`json_count` table on one resolved step, as
    `(key, table)`. Non-table entries are skipped rather than raising: this
    script is a gate, not a schema validator -- `model.rs`'s
    `deny_unknown_fields` deserialization is what rejects a malformed claim,
    and it does so with a better message."""
    out: list[tuple[str, dict]] = []
    for key in _STEP_COUNT_KEYS:
        for claim in step.get(key, []) or []:
            if isinstance(claim, dict):
                out.append((key, claim))
    return out


def resolved_steps(doc: dict) -> list[tuple[str, dict]]:
    """Every step in one parsed case file, as `(case_name, step)` -- each
    case's inline step and/or its `[[case.step]]` list, in file order. One
    traversal, shared by `assertion_strings` (which reads the steps' strings)
    and `case_count_claims` (which reads their count tables), so a case shape
    the one can see is never a shape the other silently cannot."""
    out: list[tuple[str, dict]] = []
    for case in doc.get("case", []) or []:
        if not isinstance(case, dict):
            continue
        name = case.get("name") if isinstance(case.get("name"), str) else "<unnamed>"
        inline = {k: v for k, v in case.items() if k not in _CASE_NON_STEP_KEYS}
        if inline:
            out.append((name, inline))
        step_list = case.get("step")
        if isinstance(step_list, list):
            out.extend((name, s) for s in step_list if isinstance(s, dict))
    return out


# A `${name}` placeholder, spelled exactly as `expand.rs`'s `substitute()`
# reads it: `${`, then everything up to the FIRST `}`. Same scan, same
# termination rule, so a name this finds is a name the runner would look up.
_PLACEHOLDER = re.compile(r"\$\{([^}]*)\}")


def _substituted_strings(doc: dict) -> list[str]:
    """Every string in one parsed case file that `expand.rs` actually runs
    `substitute()` over -- i.e. every place a `${NAME}` reference can be
    real rather than inert text.

    Derived from `crates/kali_case_runner/src/expand.rs`, not guessed:
    `expand()` substitutes each `[source]` KEY and VALUE (`:189-192`), and
    `substitute_step()` substitutes every string-bearing field of a step
    (`:80-146`) -- args, env keys and values, stdout/stderr and their
    `_contains`/`_absent` lists, the `json`/`fields` trees including their
    KEYS (`substitute_value`), `json_null` paths, `stdout_count` needles,
    `json_count` paths and needles, and `path`/`entry`/`body`.

    What it deliberately does NOT reach is the whole point:

    - `rationale` and a case's `name` are never substituted (`expand()`
      clones them verbatim), so a `${X}` written in prose is not a
      reference to anything.
    - `[matrix]` axis VALUES are not substituted either (`matrix_cells`
      uses them raw), so a `${X}` there is inert.
    - `[constants]` values are not substituted into each other. `bindings`
      is `file.constants` as-is and `substitute()` is single-pass -- it
      pushes a looked-up value onto `out` and never rescans it -- so
      `A = "x"` / `B = "${A}"` leaves a literal `${A}` in the expanded text
      and A is genuinely dead. This function therefore never reads the
      `[constants]` table when collecting references.

    A step's `kind` is a string this walk also sees; it is an enum spelling
    (`cli`, `browser_bundle_harness`, ...) and can never contain `${`, so
    including it costs nothing and keeps the walk a plain leaf walk rather
    than a second key whitelist that could drift from `substitute_step`."""
    out: list[str] = []

    def walk(value) -> None:
        if isinstance(value, str):
            out.append(value)
        elif isinstance(value, dict):
            for key, item in value.items():
                if isinstance(key, str):
                    out.append(key)
                walk(item)
        elif isinstance(value, list):
            for item in value:
                walk(item)

    source = doc.get("source")
    if isinstance(source, dict):
        walk(source)

    for _name, step in resolved_steps(doc):
        walk(step)

    return out


def unreferenced_constants(doc: dict) -> list[str]:
    """The `[constants]` entries of one parsed case file that expansion can
    never reach: no `${NAME}` in any substituted string, or a `[matrix]`
    axis of the same name shadowing it (`expand()` builds `bindings` from
    the constants and then `insert`s each axis over the top, so the axis
    wins and the constant is unreachable).

    WHY THIS EXISTS -- the false green it closes. `assertion_strings()` used
    to extend over every `[constants]` value whether or not anything
    referenced it, and nothing rejected an unused one. The joined haystack
    is searched by SUBSTRING, so a dead constant is a free-text channel into
    the gate rule 3 calls absolute:

        audit(nullish_assign_reject.rs, cases/nullish/assign_reject.toml)
          control                                            AUDIT OK   rc=0
          with the real `stderr_contains = ["E5506"]` deleted AUDIT FAILED rc=1
          the same deletion + an unreferenced
            `[constants] UNUSED_NOTE = "E5506"`              AUDIT OK   rc=0

    -- a genuinely dropped assertion returned to green by a constant nothing
    uses (Task 19 pilot report §12, reproduced independently by two
    reviewers and a third time by this dispatch before the fix).

    Both halves of the fix are needed and neither is redundant. Excluding a
    dead constant from `assertion_strings()` is what removes the channel;
    REPORTING it as a failure is ruling 18 #3 -- otherwise a dead constant
    and no constant at all are indistinguishable, and the next person to add
    one gets a `claim absent` message that says nothing about the constant
    sitting three lines above it. A constant expansion cannot reach has no
    legitimate use: the runner never reads it, so it can only be dead weight
    or a gate-facing one."""
    constants = doc.get("constants")
    if not isinstance(constants, dict):
        return []
    referenced: set[str] = set()
    for text in _substituted_strings(doc):
        referenced.update(_PLACEHOLDER.findall(text))
    matrix = doc.get("matrix")
    shadowed = set(matrix) if isinstance(matrix, dict) else set()
    return [name for name in constants
            if name not in referenced or name in shadowed]


def assertion_strings(doc: dict) -> list[str]:
    """Every claim-bearing string in one parsed case file: each case's
    inline step and/or `[[case.step]]` list, plus the `[constants]` values
    that expansion actually reaches.

    The `[constants]` values are here because a hoisted constant is a real
    carrier of a claim -- `switch/runtime.toml` hoists `switch_runtime.rs`'s
    `const S`/`const SS` bodies, and the source's `rule constants` claims
    are satisfiable nowhere else. But only a REFERENCED constant is a
    carrier of anything; see `unreferenced_constants` for the false green
    the unfiltered version left open, and `main` for the failure that makes
    a dead constant loud instead of merely inert."""
    out: list[str] = []

    constants = doc.get("constants")
    if isinstance(constants, dict):
        dead = set(unreferenced_constants(doc))
        out.extend(v for name, v in constants.items()
                   if isinstance(v, str) and name not in dead)

    for _name, step in resolved_steps(doc):
        out.extend(_step_assertion_strings(step))

    return out


def case_count_claims(doc: dict) -> list[dict]:
    """Every `stdout_count`/`json_count` claim in one parsed case file, as
    `{"key", "case", "needle", "path", "bound"}` -- `bound` being
    `("at_least"|"exact", K)` or `None` when the table spells neither (which
    `model.rs` rejects at parse time, so it is reported as unauditable rather
    than guessed at)."""
    out: list[dict] = []
    for name, step in resolved_steps(doc):
        for key, claim in _step_count_claims(step):
            bound: tuple[str, int] | None = None
            for word in ("at_least", "exact"):
                value = claim.get(word)
                if isinstance(value, int) and not isinstance(value, bool):
                    bound = (word, value)
                    break
            out.append(
                {
                    "key": key,
                    "case": name,
                    "needle": claim.get("needle"),
                    "path": claim.get("path"),
                    "bound": bound,
                }
            )
    return out


def _needle_correspondence(case_needle: str, variants: frozenset[str]) -> bool:
    """Does a case file's count `needle` correspond to a source count
    claim's literal (given in both of its spellings)?

    Exact equality against either spelling, except when the needle carries an
    unexpanded `${...}` reference -- a `[matrix]` axis or `[constants]` value
    substituted in at expansion time (`expand.rs` substitutes count needles
    and `json_count` paths exactly like `stdout_contains` needles). Those are
    matched as a pattern with `${...}` standing for any text, so
    `needle = "${value}\\n"` still corresponds to a source `.matches("3\\n")`
    -- and a needle with no literal text at all is handled by the caller as
    unauditable rather than trivially accepted here."""
    if "${" not in case_needle:
        return case_needle in variants
    pattern = re.compile(
        ".*".join(re.escape(part) for part in re.split(r'\$\{[^}]*\}', case_needle)),
        re.DOTALL,
    )
    return any(pattern.fullmatch(variant) for variant in variants)


def count_claim_correspondence(
    case_claims: list[dict],
    source_sites: list[tuple[str, frozenset[str], tuple[str, int] | None]],
    source_json_keys: set[str],
    source_contains: set[tuple[str, str]] | None = None,
) -> tuple[list[str], list[str], list[str]]:
    """Check every count claim the NEW files make against the OLD source --
    the reverse of every other check in this script (see the module
    docstring on why the count keys need it and the other keys don't).

    Returns `(failures, unauditable, unmirrored)`:

    - `failures`: hard failures that must fail the audit.
    - `unauditable`: claims this tool provably cannot decide, which `main`
      prints so an unaudited claim is never mistaken for an audited one.
    - `unmirrored`: SOURCE count claims that no case count claim reproduces.
      Advisory, not a failure, and the module docstring says why: measured
      against this corpus, two shipped migrations legitimately replaced a
      count claim with a STRONGER exact-`stdout` equality, which a literal-
      coverage tool cannot recognize as implying the count. Failing on those
      would report two true migrations as broken; saying nothing would leave
      "the count claim was quietly downgraded to a `stdout_contains` of the
      same needle" invisible, since the needle is present either way. So it
      prints, by name, and the reader dispositions it.
    """
    failures: list[str] = []
    unauditable: list[str] = []
    source_contains = source_contains or set()

    for claim in case_claims:
        where = f"case {claim['case']!r} {claim['key']}"
        needle = claim["needle"]
        if not isinstance(needle, str) or not needle:
            failures.append(f"{where}: claim has no `needle` string")
            continue

        if claim["key"] == "json_count":
            path = claim["path"]
            if not isinstance(path, str) or not path:
                failures.append(f"{where}: claim has no `path` string")
                continue
            for segment in path.split("."):
                if "${" in segment or not _IDENTIFIER_KEY.fullmatch(segment):
                    # An array index, or a matrix/constant reference resolved
                    # only at expansion time -- neither is a JSON key the old
                    # source could have indexed by name.
                    continue
                if segment not in source_json_keys:
                    failures.append(
                        f"{where}: `path` segment {segment!r} is not a JSON key the "
                        f"source ever indexed (path {path!r})"
                    )

        if not any(part for part in re.split(r'\$\{[^}]*\}', needle)):
            unauditable.append(
                f"{where}: needle {needle!r} is entirely a `${{...}}` reference, so "
                "there is no literal text to correspond against a source claim"
            )
            continue

        matching = [bound for _canonical, variants, bound in source_sites
                    if _needle_correspondence(needle, variants)]
        if not matching:
            # RULING 3'S AMENDED CLAUSE 4, WHICH THIS ARM USED TO MAKE
            # UNIMPLEMENTABLE (controller ruling, Task 19 batch 2).
            #
            # The amendment binds every non-browser migration: a plain
            # `.contains(x)` against a `json` string leaf becomes `json_count`
            # with `at_least = 1`, because that IS the substring form of the
            # claim -- `check_json_count` requires a string leaf and counts
            # non-overlapping occurrences, so `>= 1` is exactly "contains". An
            # exact `json.…` pin is forbidden for the same shape (it would
            # strengthen a claim the source never made that strongly).
            #
            # This arm demanded a `.matches(...).count()` site as the only
            # admissible evidence, so all three exits were closed at once: the
            # exact pin by the amendment, `json_count` by this gate, and
            # dropping the claim by rule 1. Two Task 19 batch-2 targets were
            # withdrawn on it before the controller ruled.
            #
            # TWO CONDITIONS, NOT ONE, AND THE FIRST WAS MISSING FROM THE FIRST
            # VERSION OF THIS ACCEPTANCE. The source `.contains` must be taken
            # ON THE JSON LEAF AT THE PINNED PATH -- `json_leaf_contains_sites`
            # resolves the receiver, so a `.contains` on raw stdout cannot
            # justify a `json_count`, and a `.contains` on one json leaf cannot
            # justify a claim pinned at another. Accepting any `.contains`
            # anywhere in the file, as the first version did, opened four doors
            # the reviewer demonstrated; all four are pinned as refusal tests.
            #
            # THE BOUND IS THE DISCRIMINATOR, and that is what keeps the arm's
            # strength where it was actually protecting something. `at_least = 1`
            # is the only bound a `.contains` can justify: it is a presence
            # claim and carries no number. Every other bound -- `at_least = N`
            # for N > 1, and every `exact` -- is a fidelity claim ABOUT A NUMBER,
            # which is precisely what the module docstring says forward literal
            # coverage cannot see, so those still require a genuine counting
            # site in the source and are refused without one.
            #
            # Ruling 14's shape, and its two conditions were met before this
            # shipped: no already-shipped pair's verdict moves (measured over
            # all 250 pairs by `tools/migration/audit_corpus_sweep.py --compare`),
            # and the accepted form is what the binary actually emits (the two
            # pairs it admits are live-verified trials).
            if claim["bound"] == ("at_least", 1) and claim["key"] == "json_count" \
                    and (claim["path"], needle) in source_contains:
                continue
            failures.append(
                f"{where}: needle {needle!r} corresponds to no "
                "`.matches(...).count()` assertion in the source -- a count claim "
                "the source never made (if this needle belongs to a DIFFERENT "
                "source file, audit that pair separately rather than passing both "
                "case files here)"
            )
            continue

        bound = claim["bound"]
        if bound is None:
            failures.append(f"{where}: claim sets neither `at_least` nor `exact`")
            continue
        if bound in matching:
            continue
        if all(source_bound is None for source_bound in matching):
            unauditable.append(
                f"{where}: needle {needle!r} matches a source count assertion whose "
                "comparison this script does not recognize, so its bound "
                f"({bound[0]} = {bound[1]}) is unaudited"
            )
            continue
        stated = ", ".join(
            f"{kind} {value}" for kind, value in sorted({b for b in matching if b})
        )
        failures.append(
            f"{where}: needle {needle!r} is claimed {bound[0]} = {bound[1]}, but the "
            f"source's count assertion(s) on that needle state: {stated}"
        )

    unmirrored: list[str] = []
    seen: set[tuple[str, tuple[str, int] | None]] = set()
    for canonical, variants, bound in source_sites:
        if (canonical, bound) in seen:
            continue
        seen.add((canonical, bound))
        if any(
            isinstance(claim["needle"], str)
            and _needle_correspondence(claim["needle"], variants)
            and claim["bound"] == bound
            for claim in case_claims
        ):
            continue
        spelled = f"{bound[0]} = {bound[1]}" if bound else "an unrecognized comparison"
        unmirrored.append(
            f"the source asserts {canonical!r} occurs {spelled} times, and no "
            "`stdout_count`/`json_count` claim in the case files reproduces it -- "
            "confirm it was carried by a STRONGER claim (an exact `stdout`, say) "
            "and not silently downgraded to a plain `contains` of the same needle"
        )

    return failures, unauditable, unmirrored


def load_new_docs(paths: list[Path]) -> list[tuple[Path, dict]]:
    """Parse every new case file once. Both the forward search text and the
    reverse count-claim check read these same parsed documents, so neither
    can be run against a file the other did not see."""
    docs: list[tuple[Path, dict]] = []
    for path in paths:
        try:
            docs.append((path, tomllib.loads(path.read_text())))
        except tomllib.TOMLDecodeError as error:
            print(f"error: {path}: invalid TOML: {error}", file=sys.stderr)
            raise SystemExit(2) from error
    return docs


def load_new_text(paths: list[Path]) -> str:
    """Parse every new case file and return only its assertion-bearing
    strings, joined for substring search. See the module docstring for why
    this is a parse, not a text search over the raw file."""
    pieces: list[str] = []
    for _path, doc in load_new_docs(paths):
        pieces.extend(assertion_strings(doc))
    return "\n".join(pieces)


def main() -> int:
    if len(sys.argv) < 3:
        print(__doc__, file=sys.stderr)
        return 2

    old_path = Path(sys.argv[1])
    new_paths = [Path(p) for p in sys.argv[2:]]

    old_source = old_path.read_text()

    # Resolve every submodule the top-level `.rs` pulls in, recursively,
    # via `#[path = "..."] mod ...;` or a plain `mod ...;`/`pub mod ...;`,
    # into the same claims/test-count sweep -- so auditing the top-level
    # file alone can no longer under-count its real tests, whether they're
    # one hop away or several. Purely additive to `old_source` -- a file
    # with no submodule declarations at all is unaffected.
    submodule_paths = resolve_path_mods(old_path, old_source)
    submodule_sources: list[str] = []
    for submodule_path in submodule_paths:
        if not submodule_path.is_file():
            print(
                f"error: {old_path}: a `mod` declaration (reached from "
                f"{old_path}, possibly through an intermediate submodule) "
                f"names {submodule_path}, which does not exist",
                file=sys.stderr,
            )
            return 2
        submodule_sources.append(submodule_path.read_text())
    old_source_combined = "\n".join([old_source, *submodule_sources])

    new_docs = load_new_docs(new_paths)
    new_text = "\n".join(
        piece for _path, doc in new_docs for piece in assertion_strings(doc)
    )

    # A `[constants]` entry expansion can never reach. Collected here and
    # reported below with the other verdicts; `assertion_strings` has already
    # excluded its value from `new_text`, which is what closes the false
    # green -- this arm is what makes it audible (see
    # `unreferenced_constants`).
    dead_constants: list[str] = []
    for path, doc in new_docs:
        for name in unreferenced_constants(doc):
            dead_constants.append(f"{path.name}: [constants] {name}")

    old_claims = claims(old_source_combined)

    # The reverse direction, for the count keys only (module docstring):
    # every count claim the case files make must correspond to a real
    # `.matches("lit").count()` assertion in the source, needle AND bound.
    case_claims: list[dict] = []
    for path, doc in new_docs:
        for claim in case_count_claims(doc):
            claim["case"] = f"{path.name}:{claim['case']}"
            case_claims.append(claim)
    source_sites = [
        (unquote(token), literal_variants(token), bound)
        for token, bound in count_claim_sites(old_source_combined)
    ]
    fabricated, unauditable, unmirrored = count_claim_correspondence(
        case_claims, source_sites, set(old_claims["json keys"]),
        json_leaf_contains_sites(old_source_combined),
    )

    # Rule 11's OR shape: a group of `.contains` literals the source asserts
    # DISJUNCTIVELY needs one member present, not all. See
    # `disjunctive_contains_groups` for why, and for why it fails closed.
    or_groups = disjunctive_contains_groups(old_source_combined)
    contains_variants = old_claims["contains literals"]

    def _present(canonical: str) -> bool:
        return any(v and v in new_text
                   for v in contains_variants.get(canonical, frozenset({canonical})))

    # SITE-SCOPED, not literal-scoped (controller ruling 14 / fix round 1 C1).
    # Pass 1 finds the satisfied groups and unions their `.contains` SITES;
    # pass 2 suppresses an unpinned member only when EVERY site of that literal
    # in the combined source lies inside one of those groups. A literal that is
    # also asserted unconditionally somewhere else keeps its site outside the
    # union and is still reported missing.
    all_sites = contains_sites(old_source_combined)
    satisfied: list[tuple[list[str], list[str]]] = []      # (literals, winners)
    satisfied_sites: set[int] = set()
    for group in or_groups:
        winners = [g for g in group["literals"] if _present(g)]
        if not winners:
            continue                     # every member stays a hard missing
        satisfied.append((group["literals"], winners))
        satisfied_sites |= group["sites"]

    satisfied_by_disjunction: set[str] = set()
    disjunction_notes: list[str] = []
    for literals, winners in satisfied:
        unpinned = [g for g in literals if g not in winners]
        suppressed = [g for g in unpinned
                      if all_sites.get(g, frozenset()) <= satisfied_sites]
        satisfied_by_disjunction.update(suppressed)
        note = (f"source asserts {' || '.join(repr(g) for g in literals)} as ONE "
                f"disjunctive claim (rule 11); the case files pin "
                f"{', '.join(repr(w) for w in winners)}")
        if suppressed:
            note += (f". Unpinned and NOT reported missing: "
                     f"{', '.join(repr(g) for g in suppressed)}")
        still = [g for g in unpinned if g not in suppressed]
        if still:
            note += (f". Unpinned but STILL reported missing, because the source "
                     f"also asserts it outside this disjunction: "
                     f"{', '.join(repr(g) for g in still)}")
        if not unpinned:
            note += ". Every branch is pinned; nothing is suppressed"
        disjunction_notes.append(note)

    missing: list[tuple[str, str]] = []
    # How many claims this run actually put to the case files. Not
    # `sum(len(entries))`: a claim that is blank, BORING, or suppressed by a
    # satisfied disjunction is never checked, so counting it would overstate
    # what the run demanded. See the zero-demand guard below.
    demanded = 0
    for kind, entries in old_claims.items():
        exclude = BORING.get(kind, set())
        for canonical, variants in sorted(entries.items()):
            if not canonical or canonical in exclude:
                continue
            if kind == "contains literals" and canonical in satisfied_by_disjunction:
                continue
            demanded += 1
            if not any(variant and variant in new_text for variant in variants):
                missing.append((kind, canonical))

    old_tests = sorted(set(TEST_FN.findall(old_source_combined)))
    if submodule_paths:
        names = ", ".join(str(p) for p in submodule_paths)
        print(f"{old_path}: resolved submodule(s): {names}")
    print(f"{old_path}: {len(old_tests)} #[test] fns")
    for kind, entries in old_claims.items():
        print(f"  {kind}: {len(entries)}")
    print(f"  count claims in the case files (checked back against the source): "
          f"{len(case_claims)}")
    print(f"  claims demanded of the case files: {demanded}")
    # Printed unconditionally, before any verdict: a claim this script cannot
    # decide must never be indistinguishable from one it decided in favour.
    for note in disjunction_notes:
        print(f"  DISJUNCTION — {note}")
    for note in unauditable:
        print(f"  UNAUDITED — {note}")
    for note in unmirrored:
        print(f"  NOT MIRRORED — {note}")

    # A file this script is asked to audit is, by construction, one being
    # migrated FROM -- it is expected to have real tests. Zero found (even
    # after resolving every submodule, of any shape) is never a legitimate
    # "OK"; it is either a vacuous audit of an empty/wrong file, or a
    # resolution bug in this script, and either way must not print AUDIT OK
    # (Task 18 pilot review round 1, finding 5: "0 #[test] fns / AUDIT OK"
    # was reachable simply by pointing this script at a `#[path]`-shaped
    # file before that round's fix; round 2's re-review found the same
    # shape still reachable through a PLAIN `mod` chain -- e.g.
    # `browser_cdp_smoke.rs` printed "1 #[test] fns" / "AUDIT OK",
    # examining 1 of its real 15, since the guard below only ever fires at
    # exactly zero. Both must become impossible to ship, not merely
    # discouraged).
    # EXIT CODES. 0 = AUDIT OK, 1 = AUDIT FAILED (a claim this tool CAN see
    # is missing or fabricated -- a real migration defect), 2 = a structural
    # error that stops the audit before it has a verdict (an unresolvable
    # `#[path]`), 3 = AUDIT INAPPLICABLE (the run completed but demanded
    # nothing, so it decided nothing). 3 is separate from 1 on purpose: a
    # blanket "the audit is red here, and here is why" declaration written
    # for an inapplicable pair must not also excuse a genuinely dropped
    # claim on that pair. For the same reason 1 OUTRANKS 3 where both apply
    # -- see the precedence note at the AUDIT FAILED arm below.
    if not old_tests:
        print(
            "\nAUDIT FAILED — 0 #[test] fns found (after resolving every "
            "submodule); refusing to report success against zero examined "
            "tests."
        )
        return 1

    # The same guard one step further along. The check above proves this
    # script FOUND tests; it does not prove it ASKED the case files for
    # anything. A source whose every claim is blank, BORING, or suppressed
    # leaves the `missing` loop with nothing to check, and the run prints
    # AUDIT OK having demanded zero claims -- green over nothing, reached by
    # a different route than the empty-file one. It is not hypothetical: any
    # regression in the claim extractors (a broken regex, a masking bug in
    # the raw-string scanner) degrades to exactly this shape, silently and
    # across every file at once, and the audit would keep reporting OK.
    # `case_claims` is the reverse direction and cannot substitute: it is
    # checked source-ward, so it is non-empty only when the case files
    # happen to use the count keys.
    # `case_claims` is the reverse direction -- every count claim the case
    # files make, checked back against a real source assertion. It is included
    # deliberately: a run with 0 forward claims but a non-empty reverse arm
    # DID verify something real, and calling that "inapplicable" would
    # overstate the guard. Only a run that checked nothing in EITHER direction
    # is refused.
    #
    # A REAL DEFECT OUTRANKS "THIS PAIR IS UNDECIDABLE", so the AUDIT FAILED
    # arm is tested FIRST and the rc=3 guard follows it. The two are not
    # mutually exclusive: `dead_constants` is computed from the case files
    # alone and does not need a single demanded claim to be non-empty, so a
    # pair that demands nothing AND ships a `[constants]` entry expansion can
    # never reach used to return 3 -- reporting "the audit decided nothing"
    # about a file it had in fact decided something about, and hiding the Bug 9
    # free-text channel behind the milder verdict. (`missing` and `fabricated`
    # cannot be non-empty on that path -- `missing` needs a demanded claim and
    # `fabricated` needs a `case_claims` entry -- so moving the whole block up
    # changes the verdict on exactly the dead-constant shape and on nothing
    # else. `Bug9_DeadConstantOutranksInapplicable` is the known positive.)
    if missing or fabricated or dead_constants:
        if dead_constants:
            print(
                f"\nAUDIT FAILED — {len(dead_constants)} `[constants]` "
                "entr(y/ies) that expansion can never reach (no `${NAME}` in "
                "any substituted string, or shadowed by a `[matrix]` axis of "
                "the same name). Their values are NOT counted as claim "
                "carriers; a dead constant is a free-text channel into this "
                "gate:"
            )
            for note in dead_constants:
                print(f"  [unreferenced constant] {note}")
        if missing:
            print(f"\nAUDIT FAILED — {len(missing)} claim(s) absent from the case files:")
            for kind, value in missing:
                print(f"  [{kind}] {value!r}")
        if fabricated:
            print(
                f"\nAUDIT FAILED — {len(fabricated)} count claim(s) in the case files "
                "do not correspond to a source assertion:"
            )
            for note in fabricated:
                print(f"  [count claim] {note}")
        return 1

    if not demanded and not case_claims:
        print(
            f"\nAUDIT INAPPLICABLE — {len(old_tests)} #[test] fn(s) found but "
            "0 claims demanded of the case files in either direction; refusing "
            "to report success having asserted nothing. Either the source genuinely makes no "
            "literal claim this tool can see (its assertions are exit-status "
            "or shape claims, or every literal it does carry is excluded as "
            "BORING), or claim extraction is broken. Read the pair and say "
            "which, in the case file's header, alongside whatever gate does "
            "cover it."
        )
        return 3

    print("\nAUDIT OK — every literal claim is present in the case files.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
