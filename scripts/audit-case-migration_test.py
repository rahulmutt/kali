#!/usr/bin/env python3
"""Regression suite for `scripts/audit-case-migration.py`.

This is the migration gate for the test-binary-consolidation branch: it
proves that every literal claim a hand-written `.rs` test made survives into
the TOML case file that replaces it. Across Tasks 13-17, six separate bugs
were found in the gate itself -- every one discovered because a migration
happened to trip it, every one caught by a human/reviewer rather than by the
script. This suite pins each of those regressions with a minimal input that
reproduces the original defect, plus the two structural invariants
(key-sync with `model.rs`, and "excluded-by-construction" fields staying
excluded) whose violation has already happened twice.

stdlib `unittest` only -- no `pytest`, no third-party imports. The script
under test has a hyphen in its filename, so it is loaded by path via
`importlib.util.spec_from_file_location` rather than a normal `import`.

Run directly: `python3 scripts/audit-case-migration_test.py`
"""

from __future__ import annotations

import contextlib
import importlib.util
import io
import re
import sys
import tempfile
import tomllib
import unittest
from pathlib import Path

_REPO_ROOT = Path(__file__).resolve().parent.parent
_SCRIPT_PATH = _REPO_ROOT / "scripts" / "audit-case-migration.py"
_MODEL_RS_PATH = _REPO_ROOT / "crates" / "kali_case_runner" / "src" / "model.rs"


def _load_audit_module():
    spec = importlib.util.spec_from_file_location("audit_case_migration", _SCRIPT_PATH)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


audit = _load_audit_module()


def _run_audit(old_source: str, new_toml_sources: dict, extra_files: dict | None = None) -> tuple:
    """Run `audit.main()` in-process against a temporary `old.rs` and one or
    more temporary `.toml` files (dict of filename -> contents), the same way
    `scripts/test-gate.sh`-adjacent callers invoke it as a subprocess, but
    in-process so failures point at the real traceback. Returns
    `(returncode, combined_stdout)`.

    `extra_files`, if given, is a dict of path-relative-to-the-tempdir ->
    contents, written before the audit runs -- used to exercise the
    `#[path]` submodule shape, where `old.rs` names a sibling file that must
    exist relative to `old.rs`'s own parent directory (the tempdir itself,
    here) for resolution to find it.
    """
    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)
        old_path = tmp_path / "old.rs"
        old_path.write_text(old_source)
        for rel_name, contents in (extra_files or {}).items():
            p = tmp_path / rel_name
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_text(contents)
        new_paths = []
        for name, contents in new_toml_sources.items():
            p = tmp_path / name
            p.write_text(contents)
            new_paths.append(p)

        argv = sys.argv
        sys.argv = ["audit-case-migration.py", str(old_path)] + [str(p) for p in new_paths]
        buf = io.StringIO()
        try:
            with contextlib.redirect_stdout(buf), contextlib.redirect_stderr(buf):
                rc = audit.main()
        finally:
            sys.argv = argv
        return rc, buf.getvalue()


# ---------------------------------------------------------------------------
# Bug 1: JSON_KEY reading claims out of JS fixture text.
# ---------------------------------------------------------------------------
class Bug1_JsonKeyFromFixtureText(unittest.TestCase):
    """`\\[\\s*"(\\w+)"\\s*\\]` applied to whole `.rs` text matched
    `globalThis["String"]["fromCharCode"](65)` inside a raw-string fixture,
    minting phantom claims ("String", "fromCharCode") that only a fabricated
    assertion could satisfy. Fixed by blanking raw-string spans
    (`_blank_raw_strings`) before the `JSON_KEY` scan runs."""

    def test_fixture_only_subscript_is_not_a_claim(self):
        source = (
            'fn fixture() -> &\'static str {\n'
            '    r#"globalThis["String"]["fromCharCode"](65)"#\n'
            '}\n'
        )
        result = audit.claims(source)
        self.assertNotIn("String", result["json keys"])
        self.assertNotIn("fromCharCode", result["json keys"])

    def test_genuine_assert_eq_json_index_is_a_claim(self):
        source = (
            '#[test]\n'
            'fn real_claim() {\n'
            '    assert_eq!(json["errors"][0]["code"], "E5506");\n'
            '}\n'
        )
        result = audit.claims(source)
        self.assertIn("errors", result["json keys"])
        self.assertIn("code", result["json keys"])


# ---------------------------------------------------------------------------
# Bug 2: _RAW_STRING missing its left anchor.
# ---------------------------------------------------------------------------
class Bug2_RawStringLeftAnchor(unittest.TestCase):
    """Without `(?<![A-Za-z0-9_])`, `_RAW_STRING` fired on the trailing `r`
    *inside* a plain string literal ("...operato" + "r\"" read as a raw
    string open), so the span from `operator"` to the next real `"` --
    including a genuine `json["errors"][0]["code"]` -- was blanked and
    silently dropped (measured at 93 real keys across 92 files). Pin both
    directions: the false match must not fire, and a genuine raw string must
    still be masked (so a JS/TS fixture in a raw string doesn't leak phantom
    JSON-key claims, i.e. bug 1 above)."""

    def test_word_ending_in_r_before_quote_is_not_a_raw_string_open(self):
        source = (
            'assert!(stderr.contains("unsupported operator")); '
            'assert_eq!(json["errors"][0]["code"], "E5506");'
        )
        result = audit.claims(source)
        self.assertIn("errors", result["json keys"])
        self.assertIn("code", result["json keys"])

    def test_genuine_raw_string_at_a_token_boundary_is_still_masked(self):
        # `r#"..."#` immediately after `= ` (a real token start) must still
        # be recognized and masked -- the anchor fix must not regress the
        # thing it exists to do (see Bug 1).
        source = 'let s = r#"a["b"]"#;'
        result = audit.claims(source)
        self.assertEqual(dict(result["json keys"]), {})


# ---------------------------------------------------------------------------
# Bug 3: unquote() not decoding \u{XXXX}.
# ---------------------------------------------------------------------------
class Bug3_UnquoteUnicodeEscape(unittest.TestCase):
    """`"6\\nh\\u{e9}llo"` must canonicalize to the real UTF-8 text
    `6\\nhéllo`, not the six literal characters `\\u{e9}`, or a correct
    migrated `stdout = "6\\nhéllo\\n"` could never match it."""

    def test_unicode_escape_decodes_to_real_character(self):
        token = r'"6\nh\u{e9}llo"'
        self.assertEqual(audit.unquote(token), "6\nhéllo")

    def test_double_backslash_before_u_is_not_decoded(self):
        # `"\\u{e9}"` (escaped backslash, then literal text `u{e9}`) must NOT
        # be decoded as the real unicode escape -- see unquote()'s own doc
        # comment on why the `(?<!\\)` guard must run before the `\\\\` ->
        # `\\` collapse.
        token = r'"\\u{e9}"'
        self.assertEqual(audit.unquote(token), r'\u{e9}')


# ---------------------------------------------------------------------------
# Bug 4: unquote() and the `\`+newline string continuation.
# ---------------------------------------------------------------------------
class Bug4_UnquoteLineContinuation(unittest.TestCase):
    """Rust's `\\`+newline continuation drops the backslash, the newline,
    and all following leading whitespace (` \\t\\r\\n`) up to the next
    non-whitespace byte. An escaped `\\\\` immediately before a real newline
    is NOT a continuation -- that newline is a literal embedded newline."""

    def test_continuation_eats_backslash_newline_and_leading_whitespace(self):
        # Rust source: "foo\<newline>   bar" -> "foobar"
        token = '"foo\\' + '\n' + '   bar"'
        self.assertEqual(audit.unquote(token), "foobar")

    def test_continuation_eats_tabs_and_carriage_returns_too(self):
        token = '"foo\\' + '\n' + ' \t\r\t bar"'
        self.assertEqual(audit.unquote(token), "foobar")

    def test_escaped_backslash_before_real_newline_is_not_a_continuation(self):
        # Rust source: "foo\\<newline>bar" -- `\\` is an escaped literal
        # backslash, and the newline right after it is a real embedded
        # newline in the string, not a continuation escape.
        token = '"foo\\\\' + '\n' + 'bar"'
        self.assertEqual(audit.unquote(token), "foo\\\nbar")


# ---------------------------------------------------------------------------
# Bug 5: ASSERT_EQ_VALUE_SECOND's first-argument skip.
# ---------------------------------------------------------------------------
class Bug5_AssertEqSecondArgument(unittest.TestCase):
    """The old `[^,]*` first-argument placeholder stopped at the first comma
    anywhere in the text, including one inside a nested call, so
    `assert_eq!(run_js(&src.replace("var x = 1;", "var x = 2;")), "v=100\\n")`
    registered the fixture text `"var x = 2;"` as a claim (a phantom -- it
    lives only in `[source]`-shaped fixture-construction text) and MISSED
    the real claim `"v=100\\n"`. `_assert_eq_literal_tokens`'s balanced-paren,
    string-aware argument splitter must resolve to the correct top-level
    second argument through: a nested call, a comma inside a string literal,
    and nested parens/brackets."""

    def test_nested_call_with_its_own_comma_does_not_confuse_the_split(self):
        source = (
            'assert_eq!(run_js(&src.replace("var x = 1;", "var x = 2;")), '
            '"v=100\\n");'
        )
        tokens = audit._assert_eq_literal_tokens(source)
        self.assertEqual(tokens, ['"v=100\\n"'])
        self.assertNotIn('"var x = 2;"', tokens)

    def test_comma_inside_a_string_literal_first_argument_is_not_a_split_point(self):
        source = 'assert_eq!(format!("x, y"), "expected\\n");'
        tokens = audit._assert_eq_literal_tokens(source)
        self.assertEqual(tokens, ['"expected\\n"'])

    def test_nested_brackets_in_first_argument_do_not_confuse_the_split(self):
        source = 'assert_eq!(map.get(&["a","b","c"]).unwrap(), "expected\\n");'
        tokens = audit._assert_eq_literal_tokens(source)
        self.assertEqual(tokens, ['"expected\\n"'])


# ---------------------------------------------------------------------------
# Bug 6: Char literals in the scanner.
# ---------------------------------------------------------------------------
class Bug6_CharLiteralsInScanner(unittest.TestCase):
    """`'('`, `')'`, `'{'` were treated as bare brackets, corrupting
    paren/bracket depth tracking, so
    `assert_eq!(s.replace('(', "x"), "expected\\n")` silently lost its
    second argument entirely. The fix (`_char_literal_end`) must also NOT
    mistake a Rust lifetime (`'a`, `'static`, `'_`, `&'a str`,
    `fn f<'a>(...)`) for a char literal -- a lifetime starts the same way
    (bare `'` + identifier char) but is never closed by a second `'`."""

    def test_paren_char_literal_does_not_corrupt_depth_tracking(self):
        source = 'assert_eq!(s.replace(\'(\', "x"), "expected\\n");'
        tokens = audit._assert_eq_literal_tokens(source)
        self.assertEqual(tokens, ['"expected\\n"'])

    def test_brace_and_close_paren_char_literals_also_work(self):
        for ch in ('(', ')', '{', '}', '[', ']'):
            with self.subTest(char=ch):
                source = f"assert_eq!(s.replace('{ch}', \"x\"), \"expected\\n\");"
                tokens = audit._assert_eq_literal_tokens(source)
                self.assertEqual(tokens, ['"expected\\n"'])

    def test_lifetimes_are_not_mistaken_for_char_literals(self):
        # `_char_literal_end` returns None (not a char literal) for every
        # lifetime shape -- the plausible-closing-quote window never finds
        # one, because a lifetime identifier is never followed by `'`.
        self.assertIsNone(audit._char_literal_end("'a str", 0))
        self.assertIsNone(audit._char_literal_end("'static", 0))
        self.assertIsNone(audit._char_literal_end("'_ str", 0))
        self.assertIsNone(audit._char_literal_end("&'a str", 1))
        self.assertIsNone(audit._char_literal_end("fn f<'a>(x: &'a str)", 5))

    def test_lifetimes_do_not_corrupt_a_real_assert_eq_scan(self):
        source = (
            "fn f<'a>(s: &'a str) -> &'a str {\n"
            "    assert_eq!(s.replace('(', \"x\"), \"expected\\n\");\n"
            "    s\n"
            "}\n"
        )
        tokens = audit._assert_eq_literal_tokens(source)
        self.assertEqual(tokens, ['"expected\\n"'])


# ---------------------------------------------------------------------------
# Bug 7: CONST and re.DOTALL.
# ---------------------------------------------------------------------------
class Bug7_ConstDotall(unittest.TestCase):
    """A `const NAME: &str = "a\\<newline>   b";` (continuation-wrapped rule
    literal) was invisible to `CONST` without `re.DOTALL`, because `.` does
    not match a literal newline otherwise -- not a false claim, a false
    ABSENCE: `rule constants: 0` when a real constant was declared."""

    def test_continuation_wrapped_const_is_found(self):
        source = 'const NAME: &str = "a\\' + '\n' + '   b";'
        matches = audit.CONST.findall(source)
        self.assertEqual(len(matches), 1)

    def test_continuation_wrapped_const_without_dotall_would_be_invisible(self):
        # Documents *why* the fix is needed: the same pattern compiled
        # without re.DOTALL (i.e. the pre-fix behavior) finds nothing at all
        # on a continuation-wrapped literal.
        no_dotall = re.compile(
            r'const\s+[A-Z0-9_]+\s*:\s*&str\s*=\s*\n?\s*(' + audit._STR_LITERAL + r')'
        )
        source = 'const NAME: &str = "a\\' + '\n' + '   b";'
        self.assertEqual(no_dotall.findall(source), [])


# ---------------------------------------------------------------------------
# Invariant 8: key-sync with crates/kali_case_runner/src/model.rs.
# ---------------------------------------------------------------------------
class Invariant8_ModelRsKeySync(unittest.TestCase):
    """`_STEP_LIST_KEYS`, `_STEP_SCALAR_KEYS`, `_STEP_JSON_KEYS` and
    `_STEP_COUNT_KEYS` must together cover every assertion-carrying field of
    `Step`/`RawStep` -- stated in the script's own comment above
    `_STEP_LIST_KEYS`, and violated three times (`json_null`, `stderr`, then
    `stdout_count`/`json_count`, each shipped without being added, leaving a
    new key's claims silently unaudited). This test parses `model.rs` itself
    and fails if a field exists there that this audit script neither reads
    via the four tuples nor accounts for in a named, one-line-justified
    list -- so adding a field to `model.rs` forces a deliberate decision
    here, the same way `_CASE_NON_STEP_KEYS`/`BORING` force one in the
    script itself.

    THIS TEST IS NOT SUFFICIENT ON ITS OWN, and the count keys are why. It
    proves a key is NAMED in a tuple, not that the extractor READS it: the
    one-line "fix" of adding `stdout_count`/`json_count` to `_STEP_LIST_KEYS`
    turns this test green while `assertion_strings()`' output stays
    byte-for-byte identical, because that tuple's consumer filters
    `isinstance(v, str)` and a count claim is a TOML table. That trap was
    verified on the real script by the batch-4 implementer. `CountKeyExtraction`
    below is the companion that closes it -- it drives the SAME four tuples
    but asserts on extractor output, so a key named but not read fails there.
    Neither test replaces the other; do not delete either.

    Two categories of "accounted for" that are NOT the four tuples:

    - `_NO_CLAIM_FIELDS`: exactly the exclusion list named in this task's
      brief (name, rationale, ignore, kind, path, entry, body, matrix, exit,
      and everything under [source]) -- fields that carry no literal claim
      by construction, per the audit script's own module docstring.
    - `_OTHERWISE_AUDITED_FIELDS`: fields that DO carry a claim (or gate
      structure) but are read by a separate, explicit code path rather than
      by membership in one of the three tuples: `env` (read directly in
      `_step_assertion_strings`'s `env = step.get("env")` block -- the task
      brief's exclusion list does not name it, which is a real gap between
      the brief and the code as it stands; see the report), `constants`
      (read directly in `assertion_strings`), and `case`/`step` (the
      containers `assertion_strings` iterates to reach every actual case/
      step -- structural, not claim fields in their own right).
    """

    # Fields with no literal claim, by construction -- see the audit
    # script's own module docstring for why each is safe to exclude.
    _NO_CLAIM_FIELDS = {
        "name": "case/file identifier, not an assertion",
        "rationale": "prose explaining a case, not an assertion",
        "ignore": "boolean control flag, not an assertion",
        "kind": "step-kind selector, not a literal value",
        "path": "file_json step's source path -- a file reference",
        "entry": "browser step's module entry point -- a file reference",
        "body": "browser step's inline JS body -- program text, like [source]",
        "matrix": "axis data; substituted into args/stdout_contains/etc. "
                  "before assertions are read, audited at the substitution site",
        "exit": "a real assertion (exit status) but not a string literal -- "
                "out of scope for a literal-coverage audit",
        "source": "program text ([source] fixtures), never an assertion",
    }

    # Fields that DO carry a claim (or are load-bearing structure) but are
    # read by explicit code in the audit script rather than by appearing in
    # one of the four key tuples.
    _OTHERWISE_AUDITED_FIELDS = {
        "env": "read directly in _step_assertion_strings via "
               "`env = step.get(\"env\")`, not via a _STEP_*_KEYS tuple",
        "constants": "read directly in assertion_strings via "
                     "`doc.get(\"constants\")`",
        "case": "container assertion_strings iterates (`doc.get(\"case\", [])`) "
                "to reach every actual case -- not a claim field itself",
        "step": "container assertion_strings iterates (`case.get(\"step\")`) "
                "to reach every [[case.step]] entry -- not a claim field itself",
    }

    # Not a real TOML key: `RawCase.rest` is a `#[serde(flatten)]` catch-all
    # whose *contents* are the inline step's own fields (already checked as
    # part of RawStep below). Excluded from consideration entirely, not
    # asserted about.
    _NOT_A_TOML_KEY = {"rest"}

    @staticmethod
    def _extract_struct_body(text: str, struct_name: str) -> str:
        """The text strictly between a Rust `struct NAME { ... }`'s outer
        braces, found by brace-depth counting (not a lazy-`.*?` regex) so a
        future field whose doc comment happens to contain `{`/`}` can't
        silently truncate the match."""
        marker = f"struct {struct_name} "
        start = text.find(marker)
        if start == -1:
            # Some structs in this file have no trailing space before `{`
            # variance (e.g. no derive fields) -- fall back to a looser find.
            marker = f"struct {struct_name}"
            start = text.find(marker)
        if start == -1:
            raise AssertionError(f"could not find `struct {struct_name}` in model.rs")
        open_brace = text.find("{", start)
        if open_brace == -1:
            raise AssertionError(f"could not find opening brace for struct {struct_name}")
        depth = 0
        i = open_brace
        n = len(text)
        while i < n:
            if text[i] == "{":
                depth += 1
            elif text[i] == "}":
                depth -= 1
                if depth == 0:
                    return text[open_brace + 1:i]
            i += 1
        raise AssertionError(f"unbalanced braces scanning struct {struct_name}")

    @classmethod
    def _field_names(cls, body: str) -> set:
        # A field declaration line looks like `    kind: Option<StepKind>,`
        # -- leading whitespace, an identifier, then a colon. Attribute
        # lines (`    #[serde(default)]`) don't match (they start with `#`,
        # not an identifier character), so this only picks up real fields.
        return set(re.findall(r'(?m)^\s*([a-zA-Z_][a-zA-Z0-9_]*):', body))

    def test_every_model_rs_field_is_read_by_the_audit_script_somewhere(self):
        text = _MODEL_RS_PATH.read_text()

        raw_step_fields = self._field_names(self._extract_struct_body(text, "RawStep"))
        raw_case_fields = self._field_names(self._extract_struct_body(text, "RawCase")) - self._NOT_A_TOML_KEY
        raw_case_file_fields = self._field_names(self._extract_struct_body(text, "RawCaseFile"))

        all_fields = raw_step_fields | raw_case_fields | raw_case_file_fields
        self.assertTrue(all_fields, "field extraction found nothing -- regex is broken")

        tuple_covered = (
            set(audit._STEP_LIST_KEYS)
            | set(audit._STEP_SCALAR_KEYS)
            | set(audit._STEP_JSON_KEYS)
            | set(audit._STEP_COUNT_KEYS)
        )
        accounted = tuple_covered | set(self._NO_CLAIM_FIELDS) | set(self._OTHERWISE_AUDITED_FIELDS)

        unaccounted = all_fields - accounted
        self.assertEqual(
            unaccounted,
            set(),
            f"model.rs field(s) {sorted(unaccounted)!r} are neither in one of "
            "_STEP_LIST_KEYS/_STEP_SCALAR_KEYS/_STEP_JSON_KEYS/_STEP_COUNT_KEYS "
            "nor in this test's named exclusion lists -- a new assertion-carrying "
            "field was likely added to model.rs without teaching the audit script "
            "(or this test) to read it. This is exactly the json_null/stderr/"
            "stdout_count class of bug. NOTE: naming the field in a tuple is "
            "necessary but NOT sufficient -- CountKeyExtraction below proves the "
            "extractor actually reads it.",
        )

        # Guard the guard: every name in the exclusion lists must actually
        # exist somewhere in model.rs, or the lists are stale and silently
        # hiding nothing.
        for name in self._NO_CLAIM_FIELDS:
            if name == "source":
                continue  # top-level [source] table, not a struct field name collision risk
            self.assertIn(
                name, all_fields | {"source"},
                f"exclusion-list entry {name!r} does not correspond to any field in model.rs "
                "-- stale entry, remove it",
            )
        for name in self._OTHERWISE_AUDITED_FIELDS:
            self.assertIn(
                name, all_fields,
                f"exclusion-list entry {name!r} does not correspond to any field in model.rs "
                "-- stale entry, remove it",
            )


# ---------------------------------------------------------------------------
# Invariant 9: excluded-by-construction fields stay excluded.
# ---------------------------------------------------------------------------
class Invariant9_ExcludedFieldsStayExcluded(unittest.TestCase):
    """A claim-looking literal that appears ONLY in `name`, `rationale`, a
    `#` comment, `[source]`, or `body` must NOT satisfy an audit -- the
    module docstring names four concrete ways this previously produced a
    false "every claim present" (a `rationale = \"\"\"...\"\"\"` block, an
    inline `# ...` comment, a case `name`, and a `[source]` fixture's `//
    ...` comment), all four confirmed independently sufficient on their own
    to hide a wrong diagnostic code. This end-to-end test plants the SAME
    wrong-diagnostic scenario: the real assertion in the `.rs` file expects
    `E5507`; the case file's `json` field asserts the wrong `E5506`; and
    `E5507` is scattered across every excluded-by-construction home. The
    audit must still fail."""

    _OLD_SOURCE = (
        '#[test]\n'
        'fn regression_case() {\n'
        '    assert_eq!(json["errors"][0]["code"], "E5507");\n'
        '}\n'
    )

    _HIDDEN_TOML = '''\
[[case]]
name = "E5507 regression"
rationale = """
Ensures E5507 is reported by the CLI in this exact scenario.
"""
# E5507 comment, must not satisfy the audit
body = "/* E5507 */"
json = { errors = [ { code = "E5506" } ] }

[source]
fixture = "// E5507 fixture text, not a real assertion"
'''

    def test_wrong_code_hidden_in_excluded_homes_still_fails_the_audit(self):
        rc, out = _run_audit(self._OLD_SOURCE, {"new.toml": self._HIDDEN_TOML})
        self.assertEqual(rc, 1, out)
        self.assertIn("AUDIT FAILED", out)
        self.assertIn("E5507", out)

    def test_control_same_file_with_the_correct_code_passes(self):
        # Same file, `json`'s code corrected to E5507 -- proves the failure
        # above is really about the excluded homes, not an unrelated parse
        # error or a typo in the fixture.
        fixed = self._HIDDEN_TOML.replace('"E5506"', '"E5507"')
        rc, out = _run_audit(self._OLD_SOURCE, {"new.toml": fixed})
        self.assertEqual(rc, 0, out)
        self.assertIn("AUDIT OK", out)


# ---------------------------------------------------------------------------
# End-to-end: a minimal pair that passes, and one that must fail.
# ---------------------------------------------------------------------------
class EndToEndAudit(unittest.TestCase):
    _OLD_SOURCE = (
        '#[test]\n'
        'fn minimal_case() {\n'
        '    assert!(stdout.contains("hello world"));\n'
        '    assert_eq!(json["errors"][0]["code"], "E5506");\n'
        '}\n'
    )

    def test_matching_case_file_passes(self):
        toml_source = '''\
[[case]]
name = "minimal_case"
args = ["run"]
stdout_contains = ["hello world"]
json = { errors = [ { code = "E5506" } ] }
'''
        rc, out = _run_audit(self._OLD_SOURCE, {"new.toml": toml_source})
        self.assertEqual(rc, 0, out)
        self.assertIn("AUDIT OK", out)

    def test_dropped_claim_fails(self):
        # `stdout_contains` and `json.errors[0].code` were both altered --
        # a real migration dropping/weakening claims, not a typo in the
        # test itself.
        toml_source = '''\
[[case]]
name = "minimal_case"
args = ["run"]
stdout_contains = ["hello there"]
json = { errors = [ { code = "E5507" } ] }
'''
        rc, out = _run_audit(self._OLD_SOURCE, {"new.toml": toml_source})
        self.assertEqual(rc, 1, out)
        self.assertIn("AUDIT FAILED", out)
        self.assertIn("hello world", out)
        self.assertIn("E5506", out)


# ---------------------------------------------------------------------------
# Feature: `#[path]` submodule resolution (Task 18 pilot review round 1,
# finding 5). Nine files in the browser/ family declare zero top-level
# #[test] fns and pull their real tests in via
# `#[path = "sibling_dir/child.rs"] mod name;`. Before this fix, this script
# read only the single file named on argv[1], so running it on the
# top-level file alone printed "0 #[test] fns" and then "AUDIT OK" -- a
# vacuous green that examined none of the file's real tests. This is the
# dangerous direction (a false negative: a migration could drop every
# submodule claim and this script would still say OK), the same basis as
# the six prior script fixes recorded above.
# ---------------------------------------------------------------------------
class Bug8_PathModResolution(unittest.TestCase):
    def test_top_level_file_alone_previously_reported_zero_tests(self):
        # Pins the PRE-fix failure mode directly against the CURRENT
        # (fixed) script's raw building blocks, so this test documents what
        # was wrong rather than just asserting the fix works in isolation:
        # `TEST_FN` over the un-resolved top-level source alone finds
        # nothing, because the real #[test] fns live only in the submodule.
        old_source = (
            '#[path = "old/child.rs"]\n'
            'mod child;\n'
        )
        self.assertEqual(audit.TEST_FN.findall(old_source), [])

    def test_single_path_mod_is_resolved_and_its_tests_counted(self):
        old_source = (
            '#[path = "old/child.rs"]\n'
            'mod child;\n'
        )
        child_source = (
            '#[test]\n'
            'fn only_test() {\n'
            '    assert!(stdout.contains("from child"));\n'
            '}\n'
        )
        toml_source = (
            '[[case]]\n'
            'name = "only_test"\n'
            'args = ["run"]\n'
            'stdout_contains = ["from child"]\n'
        )
        rc, out = _run_audit(
            old_source, {"new.toml": toml_source},
            extra_files={"old/child.rs": child_source},
        )
        self.assertEqual(rc, 0, out)
        self.assertIn("AUDIT OK", out)
        self.assertIn("1 #[test] fns", out)
        self.assertIn("resolved submodule", out)

    def test_multiple_path_mods_are_all_resolved(self):
        old_source = (
            '#[path = "old/run.rs"]\n'
            'mod run;\n'
            '#[path = "old/build.rs"]\n'
            'mod build;\n'
        )
        run_source = (
            '#[test]\n'
            'fn run_test() {\n'
            '    assert!(stdout.contains("run claim"));\n'
            '}\n'
        )
        build_source = (
            '#[test]\n'
            'fn build_test() {\n'
            '    assert!(stdout.contains("build claim"));\n'
            '}\n'
        )
        toml_source = (
            '[[case]]\n'
            'name = "run_test"\n'
            'args = ["run"]\n'
            'stdout_contains = ["run claim"]\n'
            '\n'
            '[[case]]\n'
            'name = "build_test"\n'
            'args = ["run"]\n'
            'stdout_contains = ["build claim"]\n'
        )
        rc, out = _run_audit(
            old_source, {"new.toml": toml_source},
            extra_files={"old/run.rs": run_source, "old/build.rs": build_source},
        )
        self.assertEqual(rc, 0, out)
        self.assertIn("AUDIT OK", out)
        self.assertIn("2 #[test] fns", out)

    def test_dropped_claim_in_a_submodule_still_fails(self):
        # The whole point: a migration that drops a claim living ONLY in the
        # submodule must still be caught, not hidden by only ever looking at
        # the (claim-free) top-level file.
        old_source = (
            '#[path = "old/child.rs"]\n'
            'mod child;\n'
        )
        child_source = (
            '#[test]\n'
            'fn only_test() {\n'
            '    assert!(stdout.contains("from child"));\n'
            '}\n'
        )
        toml_source = (
            '[[case]]\n'
            'name = "only_test"\n'
            'args = ["run"]\n'
            'stdout_contains = ["something else entirely"]\n'
        )
        rc, out = _run_audit(
            old_source, {"new.toml": toml_source},
            extra_files={"old/child.rs": child_source},
        )
        self.assertEqual(rc, 1, out)
        self.assertIn("AUDIT FAILED", out)
        self.assertIn("from child", out)

    def test_missing_submodule_file_is_a_hard_error_not_a_silent_skip(self):
        old_source = (
            '#[path = "old/missing.rs"]\n'
            'mod missing;\n'
        )
        rc, out = _run_audit(old_source, {"new.toml": '[[case]]\nname = "x"\nargs = ["run"]\n'})
        self.assertEqual(rc, 2, out)
        self.assertIn("does not exist", out)

    def test_zero_tests_after_resolution_is_audit_failed_not_audit_ok(self):
        # The brief's own Step-5 loop would have passed a completely empty
        # toml against a #[path]-shaped file before this fix (0 tests found,
        # 0 claims required, trivially "OK"). Covers both the no-#[path]-at-
        # all case AND the resolved-but-still-empty case.
        rc, out = _run_audit("fn helper_with_no_test_attribute() {}\n", {"new.toml": ""})
        self.assertEqual(rc, 1, out)
        self.assertIn("AUDIT FAILED", out)
        self.assertIn("0 #[test] fns found", out)
        self.assertNotIn("AUDIT OK", out)

        old_source = '#[path = "old/child.rs"]\nmod child;\n'
        rc2, out2 = _run_audit(
            old_source, {"new.toml": ""},
            extra_files={"old/child.rs": "fn not_a_test_either() {}\n"},
        )
        self.assertEqual(rc2, 1, out2)
        self.assertIn("AUDIT FAILED", out2)
        self.assertIn("0 #[test] fns found", out2)
        self.assertNotIn("AUDIT OK", out2)

    def test_file_with_no_path_attribute_is_unaffected(self):
        # Purely additive: a file that never used #[path] at all must audit
        # identically to before this fix.
        old_source = (
            '#[test]\n'
            'fn plain_test() {\n'
            '    assert!(stdout.contains("plain claim"));\n'
            '}\n'
        )
        toml_source = (
            '[[case]]\n'
            'name = "plain_test"\n'
            'args = ["run"]\n'
            'stdout_contains = ["plain claim"]\n'
        )
        rc, out = _run_audit(old_source, {"new.toml": toml_source})
        self.assertEqual(rc, 0, out)
        self.assertIn("AUDIT OK", out)
        self.assertNotIn("resolved submodule", out)


# ---------------------------------------------------------------------------
# Round 2 (re-review after Bug8's initial landing): `resolve_path_mods`
# handled only `#[path]`-annotated mods, one level deep. Two real corpus
# chains defeated that -- `browser_cdp_smoke.rs` reaches 14 more #[test]
# fns through a PLAIN `mod cdp_driver;` (no `#[path]` at all), and
# `inprocess.rs` reaches its CDP driver through a SECOND level of `mod`
# nesting a one-level-deep resolver never follows. Same false-negative
# class as Bug8's original finding, just via a different mod shape.
# ---------------------------------------------------------------------------
class Bug8Round2_PlainModAndNestedResolution(unittest.TestCase):
    def test_plain_mod_with_a_directory_mod_rs_is_resolved(self):
        # `mod child;` (no #[path]) with no sibling `child.rs` -- must fall
        # back to `child/mod.rs`, mirroring Rust's own resolution and the
        # real `browser_cdp_smoke.rs` -> `cdp_driver/mod.rs` shape.
        old_source = 'mod child;\n'
        child_source = (
            '#[test]\n'
            'fn only_test() {\n'
            '    assert!(stdout.contains("from child dir"));\n'
            '}\n'
        )
        toml_source = (
            '[[case]]\n'
            'name = "only_test"\n'
            'args = ["run"]\n'
            'stdout_contains = ["from child dir"]\n'
        )
        rc, out = _run_audit(
            old_source, {"new.toml": toml_source},
            extra_files={"child/mod.rs": child_source},
        )
        self.assertEqual(rc, 0, out)
        self.assertIn("AUDIT OK", out)
        self.assertIn("1 #[test] fns", out)
        self.assertIn("resolved submodule", out)

    def test_plain_mod_with_a_sibling_file_is_resolved(self):
        # The OTHER plain-mod form: `mod child;` resolving to a sibling
        # `child.rs` (tried first, before the `child/mod.rs` fallback).
        old_source = 'mod child;\n'
        child_source = (
            '#[test]\n'
            'fn only_test() {\n'
            '    assert!(stdout.contains("from sibling file"));\n'
            '}\n'
        )
        toml_source = (
            '[[case]]\n'
            'name = "only_test"\n'
            'args = ["run"]\n'
            'stdout_contains = ["from sibling file"]\n'
        )
        rc, out = _run_audit(
            old_source, {"new.toml": toml_source},
            extra_files={"child.rs": child_source},
        )
        self.assertEqual(rc, 0, out)
        self.assertIn("AUDIT OK", out)
        self.assertIn("1 #[test] fns", out)

    def test_pub_mod_is_resolved(self):
        old_source = 'pub mod child;\n'
        child_source = (
            '#[test]\n'
            'fn only_test() {\n'
            '    assert!(stdout.contains("from pub mod"));\n'
            '}\n'
        )
        toml_source = (
            '[[case]]\n'
            'name = "only_test"\n'
            'args = ["run"]\n'
            'stdout_contains = ["from pub mod"]\n'
        )
        rc, out = _run_audit(
            old_source, {"new.toml": toml_source},
            extra_files={"child.rs": child_source},
        )
        self.assertEqual(rc, 0, out)
        self.assertIn("AUDIT OK", out)
        self.assertIn("1 #[test] fns", out)

    def test_cfg_gated_plain_mod_is_resolved(self):
        # An intervening #[cfg(...)] between (nothing) and `mod name;` (the
        # reviewer's specific example) must not stop resolution.
        old_source = '#[cfg(test)]\nmod child;\n'
        child_source = (
            '#[test]\n'
            'fn only_test() {\n'
            '    assert!(stdout.contains("from cfg gated mod"));\n'
            '}\n'
        )
        toml_source = (
            '[[case]]\n'
            'name = "only_test"\n'
            'args = ["run"]\n'
            'stdout_contains = ["from cfg gated mod"]\n'
        )
        rc, out = _run_audit(
            old_source, {"new.toml": toml_source},
            extra_files={"child.rs": child_source},
        )
        self.assertEqual(rc, 0, out)
        self.assertIn("AUDIT OK", out)
        self.assertIn("1 #[test] fns", out)

    def test_cfg_gated_path_mod_is_resolved(self):
        # Same #[cfg] tolerance, but between `#[path = "..."]` and `mod`
        # (rather than plain `mod`) -- the reviewer's exact finding.
        old_source = '#[path = "elsewhere.rs"]\n#[cfg(test)]\nmod child;\n'
        child_source = (
            '#[test]\n'
            'fn only_test() {\n'
            '    assert!(stdout.contains("from cfg gated path mod"));\n'
            '}\n'
        )
        toml_source = (
            '[[case]]\n'
            'name = "only_test"\n'
            'args = ["run"]\n'
            'stdout_contains = ["from cfg gated path mod"]\n'
        )
        rc, out = _run_audit(
            old_source, {"new.toml": toml_source},
            extra_files={"elsewhere.rs": child_source},
        )
        self.assertEqual(rc, 0, out)
        self.assertIn("AUDIT OK", out)
        self.assertIn("1 #[test] fns", out)

    def test_path_mod_is_not_double_counted_as_a_plain_mod(self):
        # A `#[path = "real.rs"] mod fake;` must resolve ONLY `real.rs` --
        # not ALSO be picked up by the plain-mod pass and (wrongly) looked
        # up as `fake.rs`/`fake/mod.rs`, which don't exist here.
        old_source = '#[path = "real.rs"]\nmod fake;\n'
        real_source = (
            '#[test]\n'
            'fn only_test() {\n'
            '    assert!(stdout.contains("from real"));\n'
            '}\n'
        )
        toml_source = (
            '[[case]]\n'
            'name = "only_test"\n'
            'args = ["run"]\n'
            'stdout_contains = ["from real"]\n'
        )
        rc, out = _run_audit(
            old_source, {"new.toml": toml_source},
            extra_files={"real.rs": real_source},
        )
        self.assertEqual(rc, 0, out)
        self.assertIn("AUDIT OK", out)
        self.assertIn("resolved submodule", out)
        self.assertNotIn("fake.rs", out)

    def test_nested_two_level_chain_is_fully_resolved(self):
        # Mirrors the real `inprocess.rs` -> (#[path]) -> intermediate.rs
        # -> (plain mod) -> leaf.rs chain the reviewer named: a submodule's
        # OWN mod declarations must also be followed, not just the
        # top-level file's.
        old_source = '#[path = "mid.rs"]\nmod intermediate;\n'
        mid_source = 'mod leaf;\n'
        leaf_source = (
            '#[test]\n'
            'fn leaf_test() {\n'
            '    assert!(stdout.contains("from leaf"));\n'
            '}\n'
        )
        toml_source = (
            '[[case]]\n'
            'name = "leaf_test"\n'
            'args = ["run"]\n'
            'stdout_contains = ["from leaf"]\n'
        )
        rc, out = _run_audit(
            old_source, {"new.toml": toml_source},
            extra_files={"mid.rs": mid_source, "leaf.rs": leaf_source},
        )
        self.assertEqual(rc, 0, out)
        self.assertIn("AUDIT OK", out)
        self.assertIn("1 #[test] fns", out)
        self.assertIn("mid.rs", out)
        self.assertIn("leaf.rs", out)

    def test_dotdot_path_within_a_nested_chain_is_resolved(self):
        # Mirrors the real `inprocess/cdp_driver.rs`'s
        # `#[path = "../cdp_driver/driver.rs"]` -- a `#[path]` two levels
        # deep that climbs back OUT of the intermediate file's own
        # directory. `Path.parent / "../x.rs"` resolves correctly without
        # any `.resolve()`/normalization, but this pins it explicitly.
        old_source = 'mod sub;\n'
        sub_source = '#[path = "../climbed.rs"]\nmod climbed;\n'
        climbed_source = (
            '#[test]\n'
            'fn climbed_test() {\n'
            '    assert!(stdout.contains("climbed out"));\n'
            '}\n'
        )
        toml_source = (
            '[[case]]\n'
            'name = "climbed_test"\n'
            'args = ["run"]\n'
            'stdout_contains = ["climbed out"]\n'
        )
        rc, out = _run_audit(
            old_source, {"new.toml": toml_source},
            extra_files={"sub/mod.rs": sub_source, "climbed.rs": climbed_source},
        )
        self.assertEqual(rc, 0, out)
        self.assertIn("AUDIT OK", out)
        self.assertIn("1 #[test] fns", out)

    def test_missing_file_deep_in_a_chain_is_still_a_hard_error(self):
        old_source = 'mod sub;\n'
        sub_source = 'mod missing;\n'
        rc, out = _run_audit(
            old_source, {"new.toml": '[[case]]\nname = "x"\nargs = ["run"]\n'},
            extra_files={"sub/mod.rs": sub_source},
        )
        self.assertEqual(rc, 2, out)
        self.assertIn("does not exist", out)

    def test_self_referential_mod_does_not_hang(self):
        # A file that (nonsensically, but this is a regex-driven tool, not
        # a compiler) declares a plain `mod` with its own name, pointing
        # back at itself via #[path]. Must terminate, not loop forever.
        old_source = '#[path = "old.rs"]\nmod old;\n#[test]\nfn t() { assert!(true); }\n'
        toml_source = '[[case]]\nname = "t"\nargs = ["run"]\n'
        rc, out = _run_audit(old_source, {"new.toml": toml_source})
        self.assertEqual(rc, 0, out)
        self.assertIn("AUDIT OK", out)
        # Exactly one #[test] fn -- the self-#[path] must not double-count
        # the top-level file's own single #[test].
        self.assertIn("1 #[test] fns", out)

    def test_mutual_cycle_between_two_submodules_does_not_hang(self):
        old_source = 'mod a;\n'
        a_source = '#[path = "b.rs"]\nmod b;\n#[test]\nfn a_test() { assert!(stdout.contains("a")); }\n'
        b_source = '#[path = "a.rs"]\nmod a;\n#[test]\nfn b_test() { assert!(stdout.contains("b")); }\n'
        toml_source = (
            '[[case]]\nname = "a_test"\nargs = ["run"]\nstdout_contains = ["a"]\n'
            '\n[[case]]\nname = "b_test"\nargs = ["run"]\nstdout_contains = ["b"]\n'
        )
        rc, out = _run_audit(
            old_source, {"new.toml": toml_source},
            extra_files={"a.rs": a_source, "b.rs": b_source},
        )
        self.assertEqual(rc, 0, out)
        self.assertIn("AUDIT OK", out)
        self.assertIn("2 #[test] fns", out)

    def test_module_block_with_a_body_is_not_mistaken_for_a_file_mod(self):
        # `mod tests { ... }` (an inline module, not a file reference) must
        # NOT be resolved as if it were `mod tests;` -- no trailing `;`
        # right after the name, a `{` follows instead.
        old_source = (
            '#[cfg(test)]\n'
            'mod tests {\n'
            '    #[test]\n'
            '    fn inline_test() { assert!(true); }\n'
            '}\n'
            '#[test]\n'
            'fn top_level_test() { assert!(true); }\n'
        )
        toml_source = (
            '[[case]]\nname = "top_level_test"\nargs = ["run"]\n'
        )
        rc, out = _run_audit(old_source, {"new.toml": toml_source})
        # Must not error trying to resolve "tests.rs"/"tests/mod.rs" (which
        # don't exist) -- the inline module has no file to resolve.
        self.assertNotIn("does not exist", out)
        self.assertNotIn("resolved submodule", out)


# ---------------------------------------------------------------------------
# Round 3 (second re-review): round 2's 12 tests all put the ONLY #[test]
# behind the submodule, so the zero-test guard alone carried every one of
# them -- a mutation that disabled resolution entirely whenever the
# top-level file had ANY #[test] of its own (reproducing browser_cdp_
# smoke.rs's exact pre-fix "1 #[test] fns / AUDIT OK") still passed all 45.
# Round 3 also found PLAIN_MOD was comment-blind, string-blind, and
# unanchored (a live phantom in inprocess/cdp_driver.rs's own `//!` doc
# comment), a pathological non-terminating `..` cycle, and an unimplemented
# (and mis-documented) Rust directory-nesting rule for plain-mod children.
# ---------------------------------------------------------------------------
class Bug8Round3_MutationHardening(unittest.TestCase):
    def test_top_level_fn_and_submodule_fn_are_BOTH_examined(self):
        # THE MUST-FIX test: a top-level #[test] carrying its OWN claim,
        # PLUS a plain-mod child carrying a DIFFERENT claim. Round 2's
        # tests could not distinguish "resolution works" from "the
        # zero-test guard alone saved us," because every one of them put
        # the only #[test] behind the submodule (0 at the top level). This
        # one has 1 at the top level (non-zero, so the guard alone cannot
        # explain a passing result) and 1 more reachable only through
        # `mod child;`.
        old_source = (
            '#[test]\n'
            'fn top_level_test() {\n'
            '    assert!(stdout.contains("top level claim"));\n'
            '}\n'
            'mod child;\n'
        )
        child_source = (
            '#[test]\n'
            'fn child_test() {\n'
            '    assert!(stdout.contains("child claim"));\n'
            '}\n'
        )
        toml_full = (
            '[[case]]\nname = "top_level_test"\nargs = ["run"]\n'
            'stdout_contains = ["top level claim"]\n'
            '\n[[case]]\nname = "child_test"\nargs = ["run"]\n'
            'stdout_contains = ["child claim"]\n'
        )
        rc, out = _run_audit(
            old_source, {"new.toml": toml_full},
            extra_files={"child.rs": child_source},
        )
        self.assertEqual(rc, 0, out)
        self.assertIn("AUDIT OK", out)
        self.assertIn("2 #[test] fns", out)

        # Drop the CHILD's claim (not the top-level one) -- if resolution
        # were silently skipped (the exact mutation the reviewer applied:
        # `submodule_paths = [] if TEST_FN.findall(old_source) else
        # resolve_path_mods(...)`, which only resolves submodules when the
        # top-level file has ZERO #[test] fns of its own), the audit would
        # never even look for "child claim" and would wrongly report OK.
        toml_dropped = (
            '[[case]]\nname = "top_level_test"\nargs = ["run"]\n'
            'stdout_contains = ["top level claim"]\n'
            '\n[[case]]\nname = "child_test"\nargs = ["run"]\n'
            'stdout_contains = ["something else entirely"]\n'
        )
        rc2, out2 = _run_audit(
            old_source, {"new.toml": toml_dropped},
            extra_files={"child.rs": child_source},
        )
        self.assertEqual(rc2, 1, out2)
        self.assertIn("AUDIT FAILED", out2)
        self.assertIn("child claim", out2)

    def test_mod_mentioned_in_a_line_comment_is_not_a_phantom_declaration(self):
        # Live in this corpus: inprocess/cdp_driver.rs's own `//!` header
        # says "...resolve its unqualified `mod cdp_driver;`..." -- a
        # DOCUMENTATION mention, not a real declaration. A `//!` naming a
        # module that does not exist must not become a hard error.
        old_source = (
            '//! See also its sibling `mod ghost;` for context.\n'
            '#[test]\n'
            'fn t() { assert!(true); }\n'
        )
        toml_source = '[[case]]\nname = "t"\nargs = ["run"]\n'
        rc, out = _run_audit(old_source, {"new.toml": toml_source})
        self.assertEqual(rc, 0, out)
        self.assertNotIn("does not exist", out)
        self.assertNotIn("resolved submodule", out)

    def test_mod_mentioned_in_a_doc_comment_slash_slash_bang_is_not_a_phantom(self):
        old_source = (
            '/// mentions `mod ghost;` in its own doc text\n'
            '#[test]\n'
            'fn t() { assert!(true); }\n'
        )
        toml_source = '[[case]]\nname = "t"\nargs = ["run"]\n'
        rc, out = _run_audit(old_source, {"new.toml": toml_source})
        self.assertEqual(rc, 0, out)
        self.assertNotIn("does not exist", out)
        self.assertNotIn("resolved submodule", out)

    def test_mod_mentioned_in_a_block_comment_is_not_a_phantom(self):
        old_source = (
            '/* mod ghost; */\n'
            '#[test]\n'
            'fn t() { assert!(true); }\n'
        )
        toml_source = '[[case]]\nname = "t"\nargs = ["run"]\n'
        rc, out = _run_audit(old_source, {"new.toml": toml_source})
        self.assertEqual(rc, 0, out)
        self.assertNotIn("does not exist", out)
        self.assertNotIn("resolved submodule", out)

    def test_mod_mentioned_inside_a_plain_string_is_not_a_phantom(self):
        old_source = (
            '#[test]\n'
            'fn t() {\n'
            '    let s = "mod ghost;";\n'
            '    assert!(s.contains("ghost"));\n'
            '}\n'
        )
        toml_source = (
            '[[case]]\nname = "t"\nargs = ["run"]\nstdout_contains = ["ghost"]\n'
        )
        rc, out = _run_audit(old_source, {"new.toml": toml_source})
        self.assertEqual(rc, 0, out)
        self.assertNotIn("does not exist", out)
        self.assertNotIn("resolved submodule", out)

    def test_mod_mentioned_inside_a_raw_string_is_not_a_phantom(self):
        old_source = (
            '#[test]\n'
            'fn t() {\n'
            '    let s = r#"mod ghost;"#;\n'
            '    assert!(s.contains("ghost"));\n'
            '}\n'
        )
        toml_source = (
            '[[case]]\nname = "t"\nargs = ["run"]\nstdout_contains = ["ghost"]\n'
        )
        rc, out = _run_audit(old_source, {"new.toml": toml_source})
        self.assertEqual(rc, 0, out)
        self.assertNotIn("does not exist", out)
        self.assertNotIn("resolved submodule", out)

    def test_block_comment_open_inside_a_string_does_not_swallow_a_path_mod(self):
        # Live in this corpus, and found only by review: `package_corpus.rs`
        # :322 carries the fixture line `"./*": "./src/*.js"`, whose `/*` is
        # inside a raw string. A comment masker that is not string-aware
        # reads it as a block-comment open and blanks 13,084 characters
        # through end-of-file, so `_find_mod_declarations` returned [] for a
        # file declaring FIVE `#[path]` submodules at :754-767.
        #
        # The danger direction is a MISSED submodule, not a wrong one: a
        # runaway blanks text, it cannot mint a declaration. A parent with
        # >= 1 #[test] fn plus a silently dropped submodule audits AUDIT OK
        # over claims nobody examined. Pinned as behaviour, not as a call
        # site: any masker that gets this wrong fails here.
        old_source = (
            '#[test]\n'
            'fn parent_test() {\n'
            '    let manifest = r#"{ "exports": { "./*": "./src/*.js" } }"#;\n'
            '    assert!(manifest.contains("exports"));\n'
            '}\n'
            '\n'
            '#[path = "real/child.rs"]\n'
            'mod child;\n'
        )
        child_source = (
            '#[test]\n'
            'fn child_test() {\n'
            '    assert!(stdout.contains("claim only the child makes"));\n'
            '}\n'
        )
        toml_source = (
            '[[case]]\nname = "parent_test"\nargs = ["run"]\n'
            'stdout_contains = ["exports", "claim only the child makes"]\n'
        )
        self.assertEqual(
            audit._find_mod_declarations(old_source),
            [("real/child.rs", "child")],
            "a `/*` inside a string literal must not blank the `#[path]` that follows it",
        )
        rc, out = _run_audit(
            old_source, {"new.toml": toml_source},
            extra_files={"real/child.rs": child_source},
        )
        self.assertEqual(rc, 0, out)
        self.assertIn("2 #[test] fns", out)
        self.assertIn("child.rs", out)

    def test_line_comment_open_inside_a_string_does_not_swallow_a_path_mod(self):
        # The `//` half of the same defect: `cdp_driver/driver.rs`'s
        # `starts_with("ws://")` shape, applied to mod resolution.
        old_source = (
            '#[test]\n'
            'fn parent_test() {\n'
            '    assert!(ws_url.starts_with("ws://"), "url: {ws_url}");\n'
            '}\n'
            '#[path = "real/child.rs"]\n'
            'mod child;\n'
        )
        self.assertEqual(
            audit._find_mod_declarations(old_source),
            [("real/child.rs", "child")],
        )

    def test_path_attribute_string_survives_comment_masking(self):
        # The regression the FIRST attempt at fixing comment-blindness
        # introduced and caught before shipping: masking ALL strings
        # (including a #[path]'s own argument) before matching PATH_MOD
        # destroys the very path it needs to capture. This pins that the
        # real path is still read correctly even with comment-masking in
        # front of it.
        old_source = (
            '// a comment mentioning mod ghost; for good measure\n'
            '#[path = "real/child.rs"]\n'
            'mod fake_name;\n'
        )
        child_source = (
            '#[test]\n'
            'fn only_test() {\n'
            '    assert!(stdout.contains("from real child"));\n'
            '}\n'
        )
        toml_source = (
            '[[case]]\nname = "only_test"\nargs = ["run"]\n'
            'stdout_contains = ["from real child"]\n'
        )
        rc, out = _run_audit(
            old_source, {"new.toml": toml_source},
            extra_files={"real/child.rs": child_source},
        )
        self.assertEqual(rc, 0, out)
        self.assertIn("AUDIT OK", out)
        self.assertIn("real", out)
        self.assertIn("child.rs", out)

    def test_submod_identifier_is_not_mistaken_for_mod(self):
        # `submod` ends in the three letters "mod" but is a different
        # identifier entirely -- must not be read as `mod ghost;` was
        # somehow spelled "sub" + "mod ghost;".
        old_source = (
            '#[test]\n'
            'fn t() { assert!(true); }\n'
            'fn submod_ghost_helper() {}\n'
        )
        toml_source = '[[case]]\nname = "t"\nargs = ["run"]\n'
        rc, out = _run_audit(old_source, {"new.toml": toml_source})
        self.assertEqual(rc, 0, out)
        self.assertNotIn("does not exist", out)
        self.assertNotIn("resolved submodule", out)

    def test_dotdot_cycle_terminates_quickly_via_resolve(self):
        # A cycle spelled through a `..`-climbing #[path] that never
        # lexically repeats: `sub/mod.rs` contains
        # `#[path = "../sub/mod.rs"] mod sub;`, which re-derives a
        # longer-and-longer (but never textually identical) path string on
        # every hop without `.resolve()`. Must terminate on the first
        # semantic repeat (a handful of iterations), not after path-length
        # limits start making `is_file()` false naturally.
        old_source = 'mod sub;\n#[test]\nfn t() { assert!(true); }\n'
        sub_source = '#[path = "../sub/mod.rs"]\nmod sub;\n'
        rc, out = _run_audit(
            old_source, {"new.toml": '[[case]]\nname = "t"\nargs = ["run"]\n'},
            extra_files={"sub/mod.rs": sub_source},
        )
        self.assertEqual(rc, 0, out)
        self.assertIn("AUDIT OK", out)
        # Exactly the top-level fn -- the cycle contributes no tests of its
        # own and must not multiply "1 #[test] fns" into something else.
        self.assertIn("1 #[test] fns", out)

    def test_leaf_style_flat_sibling_nests_children_in_its_own_named_subdirectory(self):
        # Real Rust semantics: a plain `mod leaf;` resolving to an ordinary
        # sibling file `leaf.rs` (not a #[path], not a `mod.rs`) is a
        # LEAF-style module -- ITS OWN nested `mod deeper;` must resolve to
        # `leaf/deeper.rs`, a SUBDIRECTORY named after `leaf`, not a
        # coincidentally-same-named sibling file one level up. This corpus
        # uses generic submodule names (run.rs/build.rs/check.rs/...), so a
        # same-named file one directory up is a realistic collision --
        # treating every plain mod as directory-style would silently fold
        # in a foreign file's claims here (the dangerous direction).
        old_source = 'mod leaf;\n'
        leaf_source = 'mod deeper;\n'
        deeper_source_correct = (
            '#[test]\n'
            'fn deeper_test() { assert!(stdout.contains("correct deeper")); }\n'
        )
        # A DECOY at the wrong (directory-style) location -- if the bug
        # were present, this is what would get pulled in instead.
        decoy_source = (
            '#[test]\n'
            'fn decoy_test() { assert!(stdout.contains("WRONG decoy")); }\n'
        )
        toml_source = (
            '[[case]]\nname = "deeper_test"\nargs = ["run"]\n'
            'stdout_contains = ["correct deeper"]\n'
        )
        rc, out = _run_audit(
            old_source, {"new.toml": toml_source},
            extra_files={
                "leaf.rs": leaf_source,
                "leaf/deeper.rs": deeper_source_correct,
                "deeper.rs": decoy_source,
            },
        )
        self.assertEqual(rc, 0, out)
        self.assertIn("AUDIT OK", out)
        self.assertIn("1 #[test] fns", out)
        # The decoy must never have been read at all.
        self.assertNotIn("decoy", out)
        self.assertNotIn("WRONG", out)

    def test_directory_style_mod_rs_nests_children_beside_itself(self):
        # The other half of the same distinction, as a control: a
        # `name/mod.rs` (directory-style) file's OWN nested `mod` DOES
        # resolve beside itself, not into a further subdirectory --
        # matching the real `cdp_driver/mod.rs` -> `cdp_driver/driver.rs`
        # shape.
        old_source = 'mod container;\n'
        container_source = 'mod sibling;\n'
        sibling_source = (
            '#[test]\n'
            'fn sibling_test() { assert!(stdout.contains("sibling claim")); }\n'
        )
        toml_source = (
            '[[case]]\nname = "sibling_test"\nargs = ["run"]\n'
            'stdout_contains = ["sibling claim"]\n'
        )
        rc, out = _run_audit(
            old_source, {"new.toml": toml_source},
            extra_files={
                "container/mod.rs": container_source,
                "container/sibling.rs": sibling_source,
            },
        )
        self.assertEqual(rc, 0, out)
        self.assertIn("AUDIT OK", out)
        self.assertIn("1 #[test] fns", out)

    def test_path_attribute_inside_a_flat_leaf_file_still_resolves_relative_to_itself(self):
        # #[path] is NEVER subject to the leaf-vs-directory nesting
        # distinction -- confirmed against the real
        # inprocess/cdp_driver.rs -> `#[path = "../cdp_driver/driver.rs"]`
        # shape: even though cdp_driver.rs itself is leaf-style (found via
        # a plain mod from its own parent, `inprocess/browser_harness_cdp_
        # in_page_trap_propagates.rs`, itself #[path]-loaded from
        # `inprocess.rs`), its own #[path] children resolve relative to ITS
        # OWN directory, never a hypothetical leaf-nesting subdirectory.
        # Mirrors the real 3-level structure exactly: old.rs (top-level) ->
        # (plain mod) -> sub/mod.rs (directory-style) -> (plain mod) ->
        # sub/leafy.rs (leaf-style, found via the flat branch) ->
        # (#[path], climbing back OUT of sub/) -> climbed.rs (a sibling of
        # sub/, at the top level).
        old_source = 'mod sub;\n'
        sub_source = 'mod leafy;\n'
        leafy_source = '#[path = "../climbed.rs"]\nmod climbed;\n'
        climbed_source = (
            '#[test]\n'
            'fn climbed_test() { assert!(stdout.contains("climbed claim")); }\n'
        )
        toml_source = (
            '[[case]]\nname = "climbed_test"\nargs = ["run"]\n'
            'stdout_contains = ["climbed claim"]\n'
        )
        rc, out = _run_audit(
            old_source, {"new.toml": toml_source},
            extra_files={
                "sub/mod.rs": sub_source,
                "sub/leafy.rs": leafy_source,
                "climbed.rs": climbed_source,
            },
        )
        self.assertEqual(rc, 0, out)
        self.assertIn("AUDIT OK", out)
        self.assertIn("1 #[test] fns", out)


# ---------------------------------------------------------------------------
# Bug 9 (Task 18, batch 4's blocking finding): `stdout_count` / `json_count`
# were added to the case format and to the runner, and this audit script was
# never taught either side of them -- neither the case files' `needle`/`path`
# nor the source's `.matches("lit").count()` claim shape. Both sides blind
# means the gate returned the IDENTICAL result whether a count claim was
# migrated faithfully, mis-needled, mis-bounded, dropped, or invented from
# nothing.
#
# The tests below are deliberately split by WHAT they assert on, because the
# distinction is the entire finding:
#
#   - `CountKeyExtraction` asserts on `assertion_strings()`' OUTPUT. This is
#     the test whose absence created the trap: adding the two key names to
#     `_STEP_LIST_KEYS` greens `Invariant8` (which only checks tuple
#     MEMBERSHIP) while leaving the extractor's output byte-for-byte
#     unchanged, because that tuple's consumer filters `isinstance(v, str)`
#     and a count claim is a TOML table. Verified on the real script before
#     this fix. It is driven off the four `_STEP_*_KEYS` tuples themselves,
#     so it covers every present and future key, not just the two that
#     prompted it.
#   - `CountClaimSourceArm` pins the six-th source-side claim kind across
#     every spelling the corpus actually uses.
#   - `CountClaimCorrespondence` pins the reverse direction -- the only
#     direction that can see an invented or mis-bounded count claim.
# ---------------------------------------------------------------------------
class CountKeyExtraction(unittest.TestCase):
    """Every whitelisted step key must produce its planted sentinel in
    `assertion_strings()`' OUTPUT -- not merely appear in a tuple."""

    # How to build a step value carrying `sentinel`, per tuple, as
    # `(value, [every sentinel that value must produce])`. Keyed by the
    # tuple's own name so that adding a key to an existing tuple is covered
    # automatically, and adding a NEW tuple fails this test until its shape
    # is stated here. A shape with more than one string-bearing sub-field
    # gets a DISTINCT sentinel per sub-field, so half a reader (`needle`
    # read, `path` dropped) is a failure and not a pass.
    _SHAPES = {
        "_STEP_LIST_KEYS": lambda s: ([s], [s]),
        "_STEP_SCALAR_KEYS": lambda s: (s, [s]),
        "_STEP_JSON_KEYS": lambda s: ({"probe": s}, [s, "probe"]),
        "_STEP_COUNT_KEYS": lambda s: (
            [{"needle": s + "_NEEDLE", "path": s + "_PATH", "at_least": 2}],
            [s + "_NEEDLE", s + "_PATH"],
        ),
    }

    def _tuples(self):
        return {name: getattr(audit, name) for name in self._SHAPES}

    def test_every_whitelisted_key_reaches_assertion_strings_output(self):
        for tuple_name, keys in self._tuples().items():
            for key in keys:
                with self.subTest(tuple=tuple_name, key=key):
                    value, expected = self._SHAPES[tuple_name](f"SENTINEL_{key}_ZZ")
                    doc = {"case": [{"name": "c", key: value}]}
                    out = audit.assertion_strings(doc)
                    for sentinel in expected:
                        self.assertIn(
                            sentinel,
                            out,
                            f"{key!r} is named in {tuple_name} but {sentinel!r} never "
                            f"reaches assertion_strings() -- naming a key in a tuple is "
                            f"not the same as reading it (got {out!r})",
                        )

    def test_the_naive_fix_would_fail_this_test(self):
        # The exact trap, reproduced: pretend `stdout_count` were "fixed" by
        # naming it in `_STEP_LIST_KEYS` only. `_STEP_LIST_KEYS`' consumer
        # keeps `isinstance(v, str)` values, so a list of tables yields
        # nothing at all. Asserting this here means the trap can never again
        # look like a fix.
        step = {"stdout_count": [{"needle": "ZZTOP\n", "at_least": 2}]}
        list_key_only = [
            v for v in step.get("stdout_count", []) or [] if isinstance(v, str)
        ]
        self.assertEqual(
            list_key_only,
            [],
            "the _STEP_LIST_KEYS consumer is expected to discard count tables -- "
            "if this ever changes, the reasoning in CountKeyExtraction's docstring "
            "needs revisiting",
        )
        # ...whereas the real reader does extract them.
        self.assertIn("ZZTOP\n", audit._step_assertion_strings(step))

    def test_json_count_path_is_extracted_too(self):
        step = {"json_count": [{"path": "payload.stdout", "needle": "3\n", "exact": 2}]}
        out = audit._step_assertion_strings(step)
        self.assertIn("payload.stdout", out)
        self.assertIn("3\n", out)

    def test_count_keys_are_read_from_a_case_step_list_too(self):
        # Both step shapes (inline and `[[case.step]]`) must reach the same
        # reader; the inline path is the one the sentinel sweep above uses.
        doc = tomllib.loads(
            '[[case]]\nname = "c"\n'
            '[[case.step]]\nargs = ["run"]\n'
            'stdout_count = [{ needle = "STEPLIST_NEEDLE\\n", at_least = 2 }]\n'
        )
        self.assertIn("STEPLIST_NEEDLE\n", audit.assertion_strings(doc))
        self.assertEqual(len(audit.case_count_claims(doc)), 1)


class CountClaimSourceArm(unittest.TestCase):
    """The source side: `.matches("lit").count()` is a claim kind, in every
    spelling the corpus uses, with its bound read."""

    def _sites(self, source):
        return [
            (audit.unquote(token), bound)
            for token, bound in audit.count_claim_sites(source)
        ]

    def test_assert_bang_greater_equal_spelling(self):
        source = 'assert!(stdout.matches("3\\n").count() >= 2, "stdout: {stdout}");'
        self.assertEqual(self._sites(source), [("3\n", ("at_least", 2))])

    def test_assert_eq_spelling(self):
        source = (
            'assert_eq!(\n'
            '    stdout.matches("1.2649110640673518").count(),\n'
            '    6,\n'
            '    "stdout: {stdout}"\n'
            ');'
        )
        self.assertEqual(self._sites(source), [("1.2649110640673518", ("exact", 6))])

    def test_assert_eq_with_the_literal_first(self):
        source = 'assert_eq!(6, stdout.matches("x").count());'
        self.assertEqual(self._sites(source), [("x", ("exact", 6))])

    def test_assert_bang_double_equals_spelling(self):
        source = 'assert!(stdout.matches("x").count() == 4);'
        self.assertEqual(self._sites(source), [("x", ("exact", 4))])

    def test_json_branch_multiline_chain(self):
        # `browser_math_log2_log10.rs`'s real json-branch spelling: the whole
        # chain is wrapped across lines. An arm that only matched the
        # single-line form would read the raw-stdout branch of every migrated
        # helper and silently miss the json branch of the same helper.
        source = (
            'assert!(\n'
            '    json["stdout"]\n'
            '        .as_str()\n'
            '        .expect("stdout string")\n'
            '        .matches("3\\n")\n'
            '        .count()\n'
            '        >= 2,\n'
            '    "json: {json}"\n'
            ');'
        )
        self.assertEqual(self._sites(source), [("3\n", ("at_least", 2))])

    def test_needle_reaches_the_claims_surface(self):
        source = (
            '#[test]\nfn t() {\n'
            '    assert!(stdout.matches("QQ\\n").count() >= 2, "stdout: {stdout}");\n'
            '}\n'
        )
        self.assertIn("QQ\n", audit.claims(source)["count needles"])

    def test_non_literal_needle_is_not_a_claim(self):
        # `.matches(alias).count()` has no auditable literal in it at all.
        source = 'assert_eq!(source.matches(alias).count(), 2, "alias {alias}");'
        self.assertEqual(self._sites(source), [])

    def test_panic_message_arguments_are_not_scanned(self):
        # The docstring says only `assert!`'s condition (arg 0) and
        # `assert_eq!`'s two compared arguments (args 0-1) are read, but
        # nothing pinned it: widening the arity so that panic-message
        # arguments are scanned too survived the suite. Found by review.
        #
        # It is not a cosmetic scope: a format argument is diagnostic output,
        # not a claim, so reading one manufactures a claim the case file must
        # satisfy (a false AUDIT FAILED) -- and for `assert_eq!` the
        # bound-from-the-sibling-argument lookup (`args[1 - index]`) is only
        # meaningful for indices 0 and 1 in the first place.
        source = (
            'assert!(\n'
            '    ok,\n'
            '    "saw {} of them",\n'
            '    stdout.matches("MESSAGE_ONLY\\n").count()\n'
            ');\n'
            'assert_eq!(\n'
            '    a,\n'
            '    b,\n'
            '    "saw {}",\n'
            '    stdout.matches("EQ_MESSAGE_ONLY\\n").count()\n'
            ');\n'
        )
        self.assertEqual(self._sites(source), [])

    def test_count_outside_an_assertion_is_not_a_claim(self):
        # Live in `browser_math_pow_exponent_one.rs`: fixture arithmetic, not
        # a claim. Reading it as one manufactures a phantom claim that no
        # case file could satisfy (`[source]` is excluded by construction),
        # i.e. a false AUDIT FAILED on a correct migration.
        source = (
            'let expected = std::iter::repeat_n(\n'
            '    expected_value, source.matches("console.log(").count()\n'
            ').collect();'
        )
        self.assertEqual(self._sites(source), [])

    def test_retention_header_prose_is_not_a_count_claim(self):
        # This script is known to read assertion-shaped text out of `//!`
        # doc comments (the deliberately-unfixed eighth defect). This arm
        # does not inherit it, because count claims are also checked in the
        # REVERSE direction: a header quoting a count assertion would
        # otherwise manufacture a source claim for a fabricated case claim to
        # correspond to. `browser_math_asinh_acosh_atanh_identities.rs`'s
        # real header quotes exactly this shape.
        source = (
            '//! SUPERSEDED: this file retained the claim\n'
            '//!     assert!(stdout.matches("0\\n").count() >= 3, "stdout: {stdout}");\n'
            '/* assert!(stdout.matches("9\\n").count() >= 7); */\n'
            '#[test]\nfn t() { assert!(stdout.contains("x"), "s"); }\n'
        )
        self.assertEqual(self._sites(source), [])

    def test_double_slash_inside_a_string_is_not_a_comment(self):
        # Live in `cdp_driver/driver.rs`: `assert!(ws_url.starts_with("ws://"))`.
        # A comment masker that is not string-aware blanks from that `//` to
        # end-of-line, taking the literal's closing quote with it, and every
        # later string skip then runs from an unterminated string -- one
        # `assert!`'s argument text swallowed 14,561 characters of unrelated
        # code and re-minted a count claim from an assertion 70 lines away.
        # A phantom claim is the dangerous direction twice over here, since
        # the reverse check would let it legitimize a fabricated case claim.
        source = (
            'fn t() {\n'
            '    assert!(ws_url.starts_with("ws://"), "url: {ws_url}");\n'
            '    let html = "<script>console.log(\'3\');</script>";\n'
            '    assert!(stdout.matches("3\\n").count() >= 2, "stdout: {stdout}");\n'
            '}\n'
        )
        self.assertEqual(self._sites(source), [("3\n", ("at_least", 2))])

    def test_real_code_beside_a_quoting_header_is_still_read(self):
        source = (
            '//! quotes assert!(stdout.matches("9\\n").count() >= 7);\n'
            '#[test]\nfn t() {\n'
            '    assert!(stdout.matches("0\\n").count() >= 3, "s");\n'
            '}\n'
        )
        self.assertEqual(self._sites(source), [("0\n", ("at_least", 3))])

    def test_a_dropped_count_needle_fails_the_audit(self):
        old_source = (
            '#[test]\nfn t() {\n'
            '    assert!(stdout.matches("UNIQUE_COUNT_NEEDLE\\n").count() >= 2, "s");\n'
            '}\n'
        )
        toml_source = '[[case]]\nname = "t"\nargs = ["run"]\nstdout_contains = ["else"]\n'
        rc, out = _run_audit(old_source, {"new.toml": toml_source})
        self.assertEqual(rc, 1, out)
        self.assertIn("count needles", out)
        self.assertIn("UNIQUE_COUNT_NEEDLE", out)


class CountClaimCorrespondence(unittest.TestCase):
    """The reverse direction: a count claim in a case file must correspond to
    a real source count assertion, needle and bound alike."""

    _OLD_SOURCE = (
        '#[test]\n'
        'fn counted() {\n'
        '    let stdout = String::from_utf8_lossy(&output.stdout);\n'
        '    assert!(stdout.matches("3\\n").count() >= 2, "stdout: {stdout}");\n'
        '    assert!(\n'
        '        json["stdout"].as_str().expect("s").matches("3\\n").count() >= 2,\n'
        '        "json: {json}"\n'
        '    );\n'
        '}\n'
    )

    def _toml(self, body):
        return '[[case]]\nname = "counted"\nargs = ["run"]\n' + body

    def test_faithful_migration_passes(self):
        rc, out = _run_audit(
            self._OLD_SOURCE,
            {"new.toml": self._toml(
                'stdout_count = [{ needle = "3\\n", at_least = 2 }]\n'
                'json_count = [{ path = "stdout", needle = "3\\n", at_least = 2 }]\n'
            )},
        )
        self.assertEqual(rc, 0, out)
        self.assertIn("AUDIT OK", out)
        self.assertIn("count claims in the case files", out)

    def test_fabricated_needle_fails(self):
        # The batch-4 report's headline reproduction: a needle that appears
        # nowhere in the source, with a bound the source never states, used
        # to exit 0 with AUDIT OK.
        rc, out = _run_audit(
            self._OLD_SOURCE,
            {"new.toml": self._toml(
                'stdout_count = [{ needle = "3\\n", at_least = 2 }]\n'
                'json_count = [{ path = "stdout", needle = "3\\n", at_least = 2 }]\n'
            ) + (
                '\n[[case]]\nname = "fabricated"\nargs = ["run"]\n'
                'stdout_count = [{ needle = "TOTALLY_FABRICATED_NEEDLE\\n", exact = 99 }]\n'
            )},
        )
        self.assertEqual(rc, 1, out)
        self.assertIn("TOTALLY_FABRICATED_NEEDLE", out)
        self.assertIn("count claim", out)

    def test_fabricated_needle_with_a_COINCIDING_bound_still_fails(self):
        # `test_fabricated_needle_fails` above passes for TWO reasons at once
        # (its needle is invented AND its bound is one the source never
        # states), so on its own it does not exercise the needle check:
        # deleting needle correspondence entirely left the suite green.
        # Found by review. Here the bound matches a real source claim
        # exactly, so only the needle can distinguish -- an invented count
        # claim that borrows a real bound is exactly the shape a careless
        # migration produces.
        rc, out = _run_audit(
            self._OLD_SOURCE,
            {"new.toml": self._toml(
                'stdout_contains = ["3\\n", "stdout"]\n'
                'stdout_count = [{ needle = "NEVER_IN_THE_SOURCE\\n", at_least = 2 }]\n'
            )},
        )
        self.assertEqual(rc, 1, out)
        self.assertIn("NEVER_IN_THE_SOURCE", out)
        self.assertIn("corresponds to no", out)

    def test_needle_correspondence_is_equality_not_substring(self):
        # Weakening `_needle_correspondence` from equality to a substring
        # test also survived the suite. It must not: counting occurrences of
        # `"3"` is a different claim from counting occurrences of `"3\n"`
        # ("13\n23\n" holds two of the former and none of the latter), and a
        # migration that drops the newline silently changes what is counted
        # while keeping the bound.
        self.assertFalse(
            audit._needle_correspondence("3", frozenset({"3\n", "3\\n"})),
            "a needle that is merely a SUBSTRING of the source literal is a "
            "different claim, not a corresponding one",
        )
        self.assertFalse(
            audit._needle_correspondence("3\n4\n", frozenset({"3\n", "3\\n"})),
            "a needle the source literal is a substring OF is equally not a "
            "corresponding claim",
        )
        self.assertTrue(audit._needle_correspondence("3\n", frozenset({"3\n", "3\\n"})))
        rc, out = _run_audit(
            self._OLD_SOURCE,
            {"new.toml": self._toml(
                'stdout_contains = ["3\\n", "stdout"]\n'
                'stdout_count = [{ needle = "3", at_least = 2 }]\n'
            )},
        )
        self.assertEqual(rc, 1, out)
        self.assertIn("corresponds to no", out)

    def test_wrong_bound_value_fails(self):
        rc, out = _run_audit(
            self._OLD_SOURCE,
            {"new.toml": self._toml('stdout_count = [{ needle = "3\\n", at_least = 1 }]\n')},
        )
        self.assertEqual(rc, 1, out)
        self.assertIn("at_least = 1", out)
        self.assertIn("at_least 2", out)

    def test_wrong_bound_kind_fails(self):
        # `exact = 2` is not `count() >= 2`: it forbids a third occurrence
        # the source assertion allows.
        rc, out = _run_audit(
            self._OLD_SOURCE,
            {"new.toml": self._toml('stdout_count = [{ needle = "3\\n", exact = 2 }]\n')},
        )
        self.assertEqual(rc, 1, out)
        self.assertIn("exact = 2", out)

    def test_json_count_path_not_indexed_by_the_source_fails(self):
        rc, out = _run_audit(
            self._OLD_SOURCE,
            {"new.toml": self._toml(
                'json_count = [{ path = "neverIndexed", needle = "3\\n", at_least = 2 }]\n'
            )},
        )
        self.assertEqual(rc, 1, out)
        self.assertIn("neverIndexed", out)

    def test_literal_block_spelling_of_the_needle_also_corresponds(self):
        # A TOML literal block carries the escape as written; `literal_
        # variants` is why both spellings correspond to the same source
        # literal.
        rc, out = _run_audit(
            self._OLD_SOURCE,
            {"new.toml": self._toml(
                "stdout_count = [{ needle = '''3\\n''', at_least = 2 }]\n"
                # Carries `_OLD_SOURCE`'s own `json["stdout"]` key claim, which
                # is not what this test is about.
                'stdout_contains = ["stdout"]\n'
            )},
        )
        self.assertEqual(rc, 0, out)

    def test_matrix_reference_in_a_needle_is_matched_as_a_pattern(self):
        rc, out = _run_audit(
            self._OLD_SOURCE,
            {"new.toml": (
                '[matrix]\nvalue = ["3"]\n\n'
                '[[case]]\nname = "counted"\nargs = ["run"]\n'
                'stdout_count = [{ needle = "${value}\\n", at_least = 2 }]\n'
                'stdout_contains = ["3\\n", "stdout"]\n'
            )},
        )
        self.assertEqual(rc, 0, out)

    def test_wholly_substituted_needle_is_reported_unaudited_not_silently_ok(self):
        rc, out = _run_audit(
            self._OLD_SOURCE,
            {"new.toml": (
                '[matrix]\nvalue = ["3\\n"]\n\n'
                '[[case]]\nname = "counted"\nargs = ["run"]\n'
                'stdout_count = [{ needle = "${value}", at_least = 2 }]\n'
                'stdout_contains = ["3\\n", "stdout"]\n'
            )},
        )
        self.assertEqual(rc, 0, out)
        self.assertIn("UNAUDITED", out)

    def test_a_source_count_claim_no_case_claim_mirrors_is_reported_not_silent(self):
        # The forward direction is literal coverage, so a count claim
        # downgraded to a plain `stdout_contains` of the same needle still
        # passes -- the needle is present either way. It must at least be
        # SAID, or the two shipped pairs that legitimately strengthened a
        # count claim into an exact `stdout` are indistinguishable from a
        # migration that quietly dropped one.
        rc, out = _run_audit(
            self._OLD_SOURCE,
            {"new.toml": self._toml('stdout_contains = ["3\\n", "stdout"]\n')},
        )
        self.assertEqual(rc, 0, out)
        self.assertIn("NOT MIRRORED", out)
        self.assertIn("at_least = 2", out)

    def test_a_mirrored_count_claim_is_not_reported_as_unmirrored(self):
        rc, out = _run_audit(
            self._OLD_SOURCE,
            {"new.toml": self._toml(
                'stdout_count = [{ needle = "3\\n", at_least = 2 }]\n'
                'json_count = [{ path = "stdout", needle = "3\\n", at_least = 2 }]\n'
            )},
        )
        self.assertEqual(rc, 0, out)
        self.assertNotIn("NOT MIRRORED", out)

    def test_a_count_claim_against_a_source_with_no_count_assertions_fails(self):
        old_source = (
            '#[test]\nfn t() { assert!(stdout.contains("3\\n"), "s"); }\n'
        )
        rc, out = _run_audit(
            old_source,
            {"new.toml": self._toml(
                'stdout_contains = ["3\\n"]\n'
                'stdout_count = [{ needle = "3\\n", at_least = 2 }]\n'
            )},
        )
        self.assertEqual(rc, 1, out)
        self.assertIn("corresponds to no", out)


# ---------------------------------------------------------------------------
# Documented, accepted limitations -- pinned as CURRENT behavior, not as a
# desired property. A future change that accidentally alters any of these
# should show up here, not slip through silently.
# ---------------------------------------------------------------------------
class DocumentedLimitations(unittest.TestCase):
    def test_rprefix_inside_a_line_comment_is_still_read_as_a_raw_string_open(self):
        # `_RAW_STRING`'s own doc comment names this residual directly: the
        # scanner "cannot tell a genuine `r#\"` token start from the same
        # three characters appearing inside a line comment ... this is a
        # regex approximation, not a real Rust lexer." A `r#"` inside a `//`
        # comment, followed later by a `"#`-shaped closing fence, is
        # misread as a real raw-string span -- so real code (and a real
        # JSON-key claim) between them gets masked and silently dropped.
        # Documented as "not present anywhere in the corpus ... acceptable
        # for that reason, not because it is impossible in principle."
        source = (
            '// this mentions r#"weird\n'
            'fn f() {\n'
            '    assert_eq!(json["real_key"], "value");\n'
            '}\n'
            '"# more text\n'
        )
        result = audit.claims(source)
        # Current (accepted-limitation) behavior: the claim is lost. If this
        # is ever fixed, this assertion should flip to assertIn and this
        # comment should say so -- do not "fix" this test in isolation.
        self.assertNotIn("real_key", result["json keys"])

    def test_odd_backslash_run_before_a_unicode_escape_is_not_correctly_paired(self):
        # `_UNICODE_ESCAPE`'s `(?<!\\)` guard is a single-character
        # lookbehind, so it cannot do real odd/even backslash-run parity.
        # Rust semantics for source `"\\\u{e9}"` (three backslashes then
        # `u{e9}`) pair the first two backslashes into one literal `\` and
        # read the third backslash + `u{e9}` as the real escape `\u{e9}`
        # (i.e. the correct decode is one literal backslash followed by
        # `é`). The single-char lookbehind instead sees a backslash
        # immediately before the candidate and rejects it outright, so no
        # unicode decoding happens at all, and the final `\\` -> `\`
        # collapse only pairs off two of the three backslashes -- the
        # canonicalized text ends up as two literal backslashes followed by
        # the literal text `u{e9}`, not backslash+é. Not called out by name
        # in the script's comments (unlike the r#"-in-a-comment residual
        # above), but the same class of approximation, and worth pinning so
        # a change to the guard's shape is visible rather than silent.
        token = '"' + ('\\' * 3) + 'u{e9}' + '"'
        self.assertEqual(audit.unquote(token), '\\\\u{e9}')

    def test_byte_string_prefix_is_not_recognized_as_a_string_literal(self):
        # `_STR_LITERAL` is `r?#*"..."#*` -- there is no `b?`/`c?` prefix
        # handling anywhere in this script, so a hypothetical
        # `.contains(b"literal")` (a Rust byte-string literal) is not
        # matched at all. Not named in the script's comments and not
        # observed anywhere in this corpus at the time of writing (the
        # corpus only uses plain and raw string literals), but pinned here
        # as current behavior per the same "known gap, not silently
        # regressible" posture as the other two tests in this class.
        source = '.contains(b"literal")'
        self.assertEqual(audit.CONTAINS.findall(source), [])


if __name__ == "__main__":
    unittest.main()
