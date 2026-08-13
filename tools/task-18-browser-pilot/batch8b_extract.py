#!/usr/bin/env python3
r"""Structural extraction for Task 18 batch 8B's thirteen source targets.

WHY A SEPARATE MODULE, AND WHY IT PARSES RATHER THAN TRANSCRIBES.

Batch 8B migrates 375 real helper invocations out of 189 `#[test]` fns. Every
one of them names an argv, an environment, a fixture filename and an assertion
set, and every one of those is a thing an implementer can mistype. The recorded
failure mode on this project is not that a generator is wrong in an interesting
way -- it is that a hand-copied literal drifts from the source and no gate reads
prose or fixture names closely enough to notice (U8), so the case file ships
asserting something adjacent to what the source asserted.

So nothing here is transcribed. Every argv token, every `.env(...)` value, every
`.join("...")` filename and every `assert*` claim is READ OUT OF THE `.rs`, and
the extractor raises when a source stops matching the shape it models. A source
edit therefore breaks the generator instead of silently producing a case file
that describes the old source.

The one thing this module deliberately does NOT do is decide anything: which
cases share a file (U2), whether a `[matrix]` closes (rule 7), and what prose a
rationale carries are the generator's job, and they are what review has to read.

TWO SHAPES ARE MODELLED, because the thirteen targets have exactly two:

  * `flat`  -- a `#[test]` fn (or a helper it calls) builds one `Command` per
               `json_output` iteration, in a fresh tempdir. Nine targets.
  * `submod`-- a `#[path]` carrier whose `#[test]` fns live in `run.rs` and
               `test.rs` siblings, each fn building exactly one `Command`
               inline (U10). Four targets.

`assert_eq!(output.status.code(), Some(N))` and `assert!(output.status.success())`
are both read; the case format folds them into one `exit` key, and the folding is
recorded per claim rather than assumed, so a source asserting a code the case
file does not carry is a raise and not a silent drop.
"""

import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
TESTS = os.path.join(REPO, "crates/kali_cli/tests")

from lexer import find_string_literals  # noqa: E402
from case_emit import source_text, source_bytes  # noqa: E402  (8C: sources resolve from history)

HARNESS_ENV = "KALI_BROWSER_BUNDLE_HARNESS_COMMAND"


# --------------------------------------------------------------------------
# Rust chunking
# --------------------------------------------------------------------------


def _match_brace(text, open_at):
    depth, i, n = 0, open_at, len(text)
    while i < n:
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                return i
        i += 1
    raise AssertionError(f"unbalanced brace from offset {open_at}")


def fn_body(text, name):
    """`fn <name>`'s body INCLUDING its braces, plus the line it starts on."""
    m = re.search(r"\bfn\s+" + re.escape(name) + r"\s*[(<]", text)
    if not m:
        raise AssertionError(f"no `fn {name}` in this source")
    brace = text.index("{", m.end() - 1)
    end = _match_brace(text, brace)
    line = text.count("\n", 0, m.start()) + 1
    return text[brace:end + 1], line


def test_fns(text):
    """Every `#[test]` fn, in source order, with the attributes above it.

    Counted as ATTRIBUTE LINES (`line.strip() == "#[test]"`), which is the same
    predicate the batch census uses, so the generator and the census cannot
    disagree about how many tests a file has.
    """
    lines = text.split("\n")
    out = []
    for i, line in enumerate(lines):
        if line.strip() != "#[test]":
            continue
        attrs = []
        j = i - 1
        while j >= 0 and lines[j].strip().startswith("#["):
            attrs.insert(0, lines[j].strip())
            j -= 1
        k = i + 1
        m = re.match(r"\s*fn\s+([a-z0-9_]+)\s*\(", lines[k])
        if not m:
            raise AssertionError(f"`#[test]` at line {i+1} is not followed by a fn signature")
        name = m.group(1)
        body, _ = fn_body(text, name)
        out.append({"name": name, "attrs": attrs, "body": body,
                    "attr_line": i + 1, "fn_line": k + 1})
    return out


def literals(body):
    return [x["value"] for x in find_string_literals(body)]


# --------------------------------------------------------------------------
# Claim extraction
# --------------------------------------------------------------------------


_JSON_EQ = re.compile(
    r'assert_eq!\(\s*(?:json|envelope|value)((?:\[(?:"[^"]*"|[a-z_]+)\])+)\s*,\s*(.+?)\s*\)\s*;',
    re.S)
_ERR_EQ = re.compile(
    r'assert_eq!\(\s*(?:errors\[0\]|error)((?:\[(?:"[^"]*")\])+)\s*,\s*(.+?)\s*\)\s*;', re.S)


def _path_of(bracket_text, prefix=""):
    keys = re.findall(r'\["([^"]*)"\]|\[([a-z_]+)\]', bracket_text)
    parts = [a or b for a, b in keys]
    return ".".join(([prefix] if prefix else []) + parts)


def _rust_value(text):
    text = text.strip().rstrip(",")
    if text == "true":
        return True
    if text == "false":
        return False
    if re.fullmatch(r"-?\d+", text):
        return int(text)
    m = re.fullmatch(r'"((?:[^"\\]|\\.)*)"', text)
    if m:
        return m.group(1).encode().decode("unicode_escape")
    if text == "command":
        return "<command>"
    if text == "expected_origin":
        return "<expected_origin>"
    if text.startswith("serde_json::json!"):
        return _json_macro(text)
    raise AssertionError(f"cannot read Rust value {text!r}")


def _json_macro(text):
    """The VALUE inside `serde_json::json!( ... )`, parsed -- never defaulted.

    This returned a `<json-macro>` sentinel that the generator turned into `[]`.
    That is right for `assert_empty_thread_topology`'s `json!([])` and WRONG for
    the one site that spells a populated object, which shipped a case asserting
    `payload.threadTopology = []` against output that carries a live instance.
    `cargo test` caught it, which is luck: a sentinel that resolves to a
    plausible value is exactly the silent-corruption shape the audit cannot see.
    So it is parsed, and anything unparseable RAISES.
    """
    inner = text[text.index("(") + 1:]
    depth, i = 1, 0
    while i < len(inner):
        if inner[i] == "(":
            depth += 1
        elif inner[i] == ")":
            depth -= 1
            if depth == 0:
                break
        i += 1
    payload = inner[:i]
    # Rust's `json!` allows trailing commas; JSON does not.
    payload = re.sub(r",(\s*[}\]])", r"\1", payload)
    import json as _j
    try:
        return _j.loads(payload)
    except Exception as exc:
        raise AssertionError(
            f"cannot parse `serde_json::json!` payload -- refusing to guess: {exc}\n"
            f"{payload[:200]}")


def claims_in(body, *, prefix=""):
    """Every claim a body makes, as a normalised list.

    Returns a list of tuples:
      ("exit_success", bool)                -- assert!( [!] output.status.success() )
      ("exit_code", int)                    -- assert_eq!(output.status.code(), Some(N))
      ("json", dotted_path, value)          -- assert_eq!(json[...], value)
      ("json_contains", dotted_path, needle)-- assert!(json[...].as_str()...contains(x))
      ("errors_empty",)                     -- assert!(json["errors"]...is_empty())
      ("errors_nonempty",)                  -- assert!(!errors.is_empty())
      ("stdout_contains", needle)
      ("stderr_contains", needle)
      ("stderr_exact", "")
      ("stdout_empty",)
    """
    out = []
    flat = body
    if re.search(r"assert!\(\s*!\s*output\.status\.success\(\)", flat):
        out.append(("exit_success", False))
    elif re.search(r"assert!\(\s*output\.status\.success\(\)", flat):
        out.append(("exit_success", True))
    for m in re.finditer(r"assert_eq!\(\s*output\.status\.code\(\)\s*,\s*Some\((\d+)\)\s*\)", flat):
        out.append(("exit_code", int(m.group(1))))
    for m in _JSON_EQ.finditer(flat):
        path = _path_of(m.group(1), prefix)
        out.append(("json", path, _rust_value(m.group(2))))
    for m in _ERR_EQ.finditer(flat):
        path = _path_of(m.group(1), "errors.0")
        out.append(("json", path, _rust_value(m.group(2))))
    # The claim is `assert!(json["errors"]...is_empty())` -- the WHOLE assert,
    # unnegated. Matching the receiver alone made `let errors = json["errors"]
    # .as_array()...; assert!(!errors.is_empty())` -- the OPPOSITE claim, two
    # statements later -- read as "errors is empty", and every rejection helper
    # in this batch has exactly that shape.
    if re.search(r'assert!\(\s*(?:json|envelope)\["errors"\]\s*[\s\S]{0,20}?\.as_array\(\)'
                 r'[\s\S]{0,70}?\.is_empty\(\)\s*\)', flat):
        out.append(("errors_empty",))
    if re.search(r"assert!\(\s*!errors\.is_empty\(\)", flat):
        out.append(("errors_nonempty",))
    # json string-leaf `.contains`
    for m in re.finditer(
            r'(?:json|envelope)((?:\["[^"]*"\])+)\s*\.as_str\(\)[\s\S]{0,80}?'
            r'\.contains\("((?:[^"\\]|\\.)*)"\)', flat):
        out.append(("json_contains", _path_of(m.group(1), prefix),
                    m.group(2).encode().decode("unicode_escape")))
    for m in re.finditer(
            r'(?:errors\[0\]|error)\["message"\]\s*\n?\s*\.as_str\(\)[\s\S]{0,90}?'
            r'\.contains\("((?:[^"\\]|\\.)*)"\)', flat):
        out.append(("json_contains", "errors.0.message",
                    m.group(1).encode().decode("unicode_escape")))
    for m in re.finditer(r'\bmessage\s*\.contains\("((?:[^"\\]|\\.)*)"\)', flat):
        out.append(("json_contains", "errors.0.message",
                    m.group(1).encode().decode("unicode_escape")))
    for m in re.finditer(r'\bstdout\s*\.contains\("((?:[^"\\]|\\.)*)"\)', flat):
        out.append(("stdout_contains", m.group(1).encode().decode("unicode_escape")))
    for m in re.finditer(r'\bstderr\s*\.contains\("((?:[^"\\]|\\.)*)"\)', flat):
        out.append(("stderr_contains", m.group(1).encode().decode("unicode_escape")))
    if re.search(r'assert_eq!\(\s*String::from_utf8_lossy\(&output\.stderr\)\s*,\s*""\s*\)', flat):
        out.append(("stderr_exact", ""))
    if re.search(r'assert!\(\s*String::from_utf8_lossy\(&output\.stdout\)\.is_empty\(\)', flat):
        out.append(("stdout_empty",))
    return out


def or_groups(body):
    """`assert!(a.contains(X) || a.contains(Y))` -- rule 11's disjunctions.

    Returned as a list of lists of needles. Modelled explicitly because the
    case format has no disjunction and ruling 17 needs to know which needles
    were alternatives rather than separate claims: pinning all of them is a
    rule-2 invention, so the generator must be able to tell them apart.
    """
    groups = []
    for m in re.finditer(
            r'assert!\(\s*((?:[a-z_]+\.contains\("(?:[^"\\]|\\.)*"\)\s*\|\|\s*)+'
            r'[a-z_]+\.contains\("(?:[^"\\]|\\.)*"\))', flat_ws(body)):
        needles = [x.encode().decode("unicode_escape")
                   for x in re.findall(r'\.contains\("((?:[^"\\]|\\.)*)"\)', m.group(1))]
        groups.append(needles)
    return groups


def flat_ws(text):
    return re.sub(r"\s+", " ", text)


# --------------------------------------------------------------------------
# Comment blocks (rule 12) and doc comments (rule 13)
# --------------------------------------------------------------------------


def comment_blocks(text, kind="//"):
    """Contiguous `//` (or `///`) comment blocks, as (first_line, [lines]).

    `///` and `//` are separated because rule 12 and rule 13 attribute
    differently: a `///` block belongs to the item it documents (and so to the
    cases whose call chain reaches that item), while a bare `//` block is
    file-or-section prose. Reading them with one predicate is how a generator
    ends up carrying a helper's doc into cases that never call it (U6's
    over-attribution, explicitly forbidden).
    """
    out, cur, start = [], [], None
    for i, line in enumerate(text.split("\n"), start=1):
        s = line.strip()
        is_doc = s.startswith("///")
        is_plain = s.startswith("//") and not is_doc and not s.startswith("//!")
        want = is_doc if kind == "///" else is_plain
        if want:
            if start is None:
                start = i
            cur.append(s.lstrip("/").strip())
        else:
            if cur:
                out.append((start, cur))
            cur, start = [], None
    if cur:
        out.append((start, cur))
    return out


def doc_owner(text, first_line):
    """The `fn` name a `///` block at `first_line` documents."""
    lines = text.split("\n")
    i = first_line - 1
    while i < len(lines):
        m = re.match(r"\s*(?:pub\s+)?fn\s+([a-z0-9_]+)", lines[i])
        if m:
            return m.group(1)
        if not lines[i].strip().startswith("//"):
            break
        i += 1
    raise AssertionError(f"no fn follows the doc block starting at line {first_line}")


def prose(block):
    return " ".join(x for x in block[1] if x).strip()


# --------------------------------------------------------------------------
# The `submod` shape -- the four runtime_summary_fallback carriers
# --------------------------------------------------------------------------


def submodules(carrier_text, stem):
    """The `#[path]` siblings a carrier declares, derived from the carrier.

    Derived rather than assumed (`{run,test}.rs` guessed from the stem) because
    a guessed directory name is how batch 8A shipped an arm that was dead while
    looking correct: the blob is `<stem>.rs` while its `#[path]` names
    `browser_<stem>/...`.
    """
    paths = re.findall(r'#\[path\s*=\s*"([^"]+)"\]', carrier_text)
    if not paths:
        raise AssertionError(f"{stem}: no `#[path]` declaration -- not a submodule carrier")
    for p in paths:
        if not p.startswith(f"browser_{stem}/"):
            raise AssertionError(
                f"{stem}: `#[path]` names {p!r}, which is not under browser_{stem}/")
    return paths


def read_submod(stem, rel):
    # 8C: the sibling directory went with its carrier (U10), so a submodule is
    # resolved from history exactly as the carrier is -- one reader, one answer.
    return source_bytes(rel)


def summary_fallback_rows(stem):
    """One row per `#[test]` fn across the carrier's two submodules.

    A row is everything the case file needs and nothing it does not:
    the fn name, its `#[cfg]` attributes, the fixture filename, the harness
    command the source hands `KALI_BROWSER_BUNDLE_HARNESS_COMMAND`, the argv,
    the assertion helper it routes its output through, and its own inline
    claims.
    """
    carrier = source_text(stem, quiet=True)   # 8C: deleted source
    subs = submodules(carrier, stem)
    writer = re.search(r"fn (write_\w+_source)\(", carrier).group(1)
    fixture_body = literals(fn_body(carrier, writer)[0])
    fixture_body = [v for v in fixture_body if v.startswith("Kali.test(")]
    assert len(fixture_body) == 1, f"{stem}: {len(fixture_body)} candidate fixture bodies"
    fixture_body = fixture_body[0]

    rows = []
    for rel in subs:
        half = os.path.basename(rel)[:-3]
        text = read_submod(stem, rel)
        for fn in test_fns(text):
            b = fn["body"]
            names = re.findall(r'\.join\("([^"]+)"\)', b)
            assert len(names) == 1, (stem, fn["name"], names)
            hcs = [v for v in literals(b) if v.startswith("node ")]
            assert len(hcs) == 1, (stem, fn["name"], len(hcs))
            argv = re.findall(r'\.arg\("([^"]*)"\)', b)
            helper = None
            for h in ("assert_browser_summary_json_failed", "assert_browser_summary_json",
                      "parse_failed_json_stdout", "parse_json_stdout"):
                if re.search(r"\b" + h + r"\(&output\)", b):
                    helper = h
                    break
            rows.append({
                "stem": stem, "half": half, "sub": rel, "fn": fn["name"],
                "attrs": fn["attrs"], "file": names[0], "harness": hcs[0],
                "argv": argv + [names[0]], "helper": helper,
                "claims": claims_in(b), "fn_line": fn["fn_line"],
                "writer": writer, "fixture": fixture_body,
                "sub_text": text, "carrier_text": carrier,
            })
    return rows, carrier, subs, writer, fixture_body


def helper_claims(carrier_text, helper):
    """What a carrier-level assertion helper asserts, read out of its body."""
    if helper is None:
        return []
    body, _ = fn_body(carrier_text, helper)
    return claims_in(body)


if __name__ == "__main__":
    for stem in ("runtime_summary_fallback_js_input", "runtime_summary_fallback_jsx_input",
                 "runtime_summary_fallback_ts_input", "runtime_summary_fallback_tsx_input"):
        rows, carrier, subs, writer, fx = summary_fallback_rows(stem)
        by = {}
        for r in rows:
            by[r["half"]] = by.get(r["half"], 0) + 1
        print(f"{stem}: {by}  subs={subs}  writer={writer}")


# --------------------------------------------------------------------------
# The `flat` shape -- call-site enumeration with loops expanded (rule 7)
# --------------------------------------------------------------------------


def _resolve_arg(rs_text, token, loopvars):
    token = token.strip().rstrip(",")
    if token in loopvars:
        return ("var", token)
    if token in ("true", "false"):
        return ("lit", token == "true")
    m = re.fullmatch(r'"((?:[^"\\]|\\.)*)"', token, re.S)
    if m:
        return ("lit", m.group(1).encode().decode("unicode_escape"))
    m = re.fullmatch(r'r#"(.*?)"#', token, re.S)
    if m:
        return ("lit", m.group(1))
    m = re.fullmatch(r'r"(.*?)"', token, re.S)
    if m:
        return ("lit", m.group(1))
    m = re.fullmatch(r"([a-z0-9_]+)\(\)", token)
    if m:
        vals = literals(fn_body(rs_text, m.group(1))[0])
        if len(vals) != 1:
            raise AssertionError(f"`{m.group(1)}()` returns {len(vals)} literals, wanted 1")
        return ("lit", vals[0])
    m = re.fullmatch(r"&\[\]", token)
    if m:
        return ("lit", [])
    m = re.fullmatch(r'&\[((?:\s*"(?:[^"\\]|\\.)*"\s*,?)+)\]', token, re.S)
    if m:
        return ("lit", [x.encode().decode("unicode_escape")
                        for x in re.findall(r'"((?:[^"\\]|\\.)*)"', m.group(1))])
    raise AssertionError(f"cannot resolve call argument {token!r}")


def _split_args(text):
    out, depth, cur, i = [], 0, "", 0
    instr = None
    while i < len(text):
        c = text[i]
        if instr:
            cur += c
            if text.startswith(instr, i):
                i += len(instr)
                cur += text[i - len(instr) + 1:i]
                instr = None
                continue
            if c == "\\" and instr == '"':
                cur += text[i + 1]
                i += 2
                continue
            i += 1
            continue
        if text.startswith('r#"', i):
            instr = '"#'
            cur += 'r#"'
            i += 3
            continue
        if c == '"':
            instr = '"'
            cur += c
            i += 1
            continue
        if c in "([{":
            depth += 1
        elif c in ")]}":
            depth -= 1
        if c == "," and depth == 0:
            out.append(cur)
            cur = ""
            i += 1
            continue
        cur += c
        i += 1
    if cur.strip():
        out.append(cur)
    return [x.strip() for x in out if x.strip()]


def invocations(rs_text, body, helper_names):
    """Every real helper call in `body`, with every `for x in [...]` expanded.

    Rule 7's precondition ("enumerate every real helper invocation in the
    source, expanding every loop") is a computation, not an observation, so it
    is done here and the arithmetic is asserted against it rather than counted
    by hand.
    """
    loops = []
    for m in re.finditer(r"for\s+([a-z_]+)\s+in\s+\[([^\]]*)\]\s*\{", body, re.S):
        values = [x.encode().decode("unicode_escape")
                  for x in re.findall(r'"((?:[^"\\]|\\.)*)"', m.group(2))]
        if not values:
            values = [x == "true" for x in re.findall(r"\b(true|false)\b", m.group(2))]
        loops.append((m.group(1), values))
    loopvars = {name for name, _ in loops}

    calls = []
    for helper in helper_names:
        for m in re.finditer(re.escape(helper) + r"\s*\(", body):
            open_at = m.end() - 1
            close = _match_brace_paren(body, open_at)
            args = [_resolve_arg(rs_text, a, loopvars)
                    for a in _split_args(body[open_at + 1:close])]
            calls.append((m.start(), helper, args))
    calls.sort()

    out = []
    import itertools
    combos = [dict(zip([n for n, _ in loops], vals))
              for vals in itertools.product(*[v for _, v in loops])] or [{}]
    for combo in combos:
        for _, helper, args in calls:
            resolved = []
            for kind, value in args:
                resolved.append(combo[value] if kind == "var" else value)
            out.append((helper, resolved))
    return out


def _match_brace_paren(text, open_at):
    depth, i, instr = 0, open_at, None
    while i < len(text):
        c = text[i]
        if instr:
            if text.startswith(instr, i):
                i += len(instr)
                instr = None
                continue
            if c == "\\" and instr == '"':
                i += 2
                continue
            i += 1
            continue
        if text.startswith('r#"', i):
            instr, i = '"#', i + 3
            continue
        if c == '"':
            instr, i = '"', i + 1
            continue
        if c == "(":
            depth += 1
        elif c == ")":
            depth -= 1
            if depth == 0:
                return i
        i += 1
    raise AssertionError("unbalanced paren")


ARGV_ALIASES = {"&source_path": "@entry", "&policy_path": "kali.policy.json"}


def argv_of(rs_text, body, bind, *, entry, resolve):
    """The argv a body builds, in source order, under `bind`.

    DERIVED, NOT WRITTEN DOWN. `ARGV ORDER` is a sentence every case file in
    this family carries, and the only way for it to stay true across 375
    invocations is for the order to come out of the `.arg(...)` chain itself.
    A token this cannot resolve raises: a silently dropped argument is a
    different command under the same case name.
    """
    body = resolve(body, bind)
    # `for arg in command_args { cli.arg(arg); }` -- expand against the binding.
    def _expand(m):
        var, listname = m.group(1), m.group(2)
        values = bind.get(listname)
        if values is None:
            raise AssertionError(f"argv loop over unbound `{listname}`")
        inner = m.group(3)
        if f".arg({var})" not in inner.replace(" ", ""):
            raise AssertionError(f"argv loop body does not `.arg({var})`")
        return "".join(f'.arg("{v}")' for v in values)
    body = re.sub(r"for\s+([a-z_]+)\s+in\s+([a-z_]+)\s*\{([^{}]*)\}", _expand, body)

    out = []
    for m in re.finditer(r"\.arg\(\s*([^)]*?)\s*\)", body):
        tok = m.group(1).strip()
        if tok in ARGV_ALIASES:
            tok = ARGV_ALIASES[tok]
            out.append(entry if tok == "@entry" else tok)
            continue
        lit = re.fullmatch(r'"((?:[^"\\]|\\.)*)"', tok)
        if lit:
            out.append(lit.group(1).encode().decode("unicode_escape"))
            continue
        if tok in bind and isinstance(bind[tok], str):
            out.append(bind[tok])
            continue
        raise AssertionError(f"cannot resolve `.arg({tok})` -- bind it or alias it")
    return out


def env_of(body):
    """The `.env(...)` / `.env_remove(...)` a body applies, as (name, value|None)."""
    out = []
    for m in re.finditer(
            r'\.env\(\s*(?:kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV|'
            r'"KALI_BROWSER_BUNDLE_HARNESS_COMMAND")\s*,\s*"((?:[^"\\]|\\.)*)"\s*\)', body):
        out.append((HARNESS_ENV, m.group(1).encode().decode("unicode_escape")))
    for m in re.finditer(
            r'\.env_remove\(\s*(?:kali_runtime_contract::BROWSER_HARNESS_COMMAND_ENV|'
            r'"KALI_BROWSER_BUNDLE_HARNESS_COMMAND")\s*\)', body):
        out.append((HARNESS_ENV, None))
    if len(out) > 1:
        raise AssertionError(f"body applies {len(out)} harness-env operations")
    return out[0] if out else None


def fn_params(text, name):
    """The parameter names of `fn <name>`, in order.

    Used to bind a call site's literal arguments to the callee's parameters, so
    a helper's branches resolve under the SAME names the source spells. Reading
    the signature rather than hardcoding the order is what makes a reordered or
    renamed parameter a generator error instead of a silently mis-bound case.
    """
    m = re.search(r"\bfn\s+" + re.escape(name) + r"\s*\(", text)
    if not m:
        raise AssertionError(f"no `fn {name}` in this source")
    close = _match_brace_paren(text, m.end() - 1)
    params = []
    for part in _split_args(text[m.end():close]):
        pm = re.match(r"([a-z_][a-z0-9_]*)\s*:", part)
        if pm:
            params.append(pm.group(1))
    return params


def json_loop(body):
    """True when a body loops `for json_output in [false, true]` itself."""
    return bool(re.search(r"for\s+json_output\s+in\s+\[\s*false\s*,\s*true\s*\]", body))


def entry_of(rs_text, body, bind, resolve):
    """The fixture filename a body writes, resolved under `bind`."""
    body = resolve(body, bind)
    hits = re.findall(r'\.join\(\s*([^)]*?)\s*\)', body)
    names = []
    for tok in hits:
        if tok in ("\"kali.json\"", "\"kali.policy.json\""):
            continue
        lit = re.fullmatch(r'"((?:[^"\\]|\\.)*)"', tok)
        if lit:
            names.append(lit.group(1).encode().decode("unicode_escape"))
        elif tok in bind and isinstance(bind[tok], str):
            names.append(bind[tok])
    if len(set(names)) != 1:
        raise AssertionError(f"expected exactly one entry filename, got {names}")
    return names[0]


def fixture_of(rs_text, body, bind, resolve):
    """The program text a body writes to its entry, resolved under `bind`."""
    resolved = resolve(body, bind)
    m = re.search(r"fs::write\(\s*&source_path\s*,\s*([\s\S]*?)\)\s*\.expect", resolved)
    if not m:
        raise AssertionError("no `fs::write(&source_path, ...)` in this body")
    expr = m.group(1).strip().rstrip(",")
    if expr in bind and isinstance(bind[expr], str):
        return bind[expr]
    if re.fullmatch(r'r#"[\s\S]*"#|"[\s\S]*"', expr):
        vals = literals(expr)
        if len(vals) == 1:
            return vals[0]
    # `let source = <literal>;` -- the FIRST literal after the binder, not the
    # text up to the first `;`. A raw string here legitimately contains `;`
    # (`Kali.test('...', () => { ... });`), so scanning to a semicolon splits
    # the fixture in half and the lexer then reports an unterminated raw string.
    m2 = re.search(r"let\s+" + re.escape(expr) + r"\s*=", resolved)
    if m2:
        vals = literals(resolved[m2.end():])
        if vals:
            return vals[0]
    m3 = re.fullmatch(r"([a-z0-9_]+)\(\)", expr)
    if m3:
        vals = literals(fn_body(rs_text, m3.group(1))[0])
        if len(vals) == 1:
            return vals[0]
    raise AssertionError(f"cannot resolve the program written to the entry: {expr!r}")


def writes_manifest(rs_text, body, bind, resolve):
    return "write_browser_api_surface_manifest(" in resolve(body, bind)


def manifest_body(rs_text):
    vals = literals(fn_body(rs_text, "write_browser_api_surface_manifest")[0])
    hits = [v for v in vals if v.lstrip().startswith("{")]
    if len(hits) != 1:
        raise AssertionError(f"{len(hits)} candidate manifest bodies")
    return hits[0]


def policy_body(rs_text):
    vals = literals(fn_body(rs_text, "write_valid_policy")[0])
    hits = [v for v in vals if v.lstrip().startswith("{")]
    if len(hits) != 1:
        raise AssertionError(f"{len(hits)} candidate policy bodies")
    return hits[0]
