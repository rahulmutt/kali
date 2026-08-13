#!/usr/bin/env python3
r"""Shape-strict extractor for Task 19 batch 4's TEXT-CLI multi-builder family.

WHAT THIS IS AND WHY IT EXISTS (U12). Batch 3's targets shared ONE mechanical
shape -- `fn run_source(src) -> Output`, called once per `#[test]` -- and its
extractor could therefore be closed over that one call. Of the 19 CLEAN targets
left at BASE exactly one is that shape. The rest are MULTI-BUILDER and
MULTI-MODE: 26 fixture-builder fns in one file, four `assert_*` helpers in
another, `#[test]` fns that loop over extension lists, a helper that branches on
a call-site-constant parameter, and 14 inline `Command::new` sites.

So this is not a pattern-matcher over one call; it is a small INTERPRETER over a
closed statement/expression language, and every construct outside that language
RAISES.

  **CLOSURE IS OVER CLAIMS, NOT OVER ASSERTION MACROS.**

Batch 3's table was closed over `assert*!` and therefore silently skipped three
real claim shapes (a claim written as `if !x.contains(y) { panic!() }`, a claim
carried by `.expect()`, and any claim outside a macro); its fix round 1 added
`residual_claims` for exactly that. That closure is the STARTING point here, not
an afterthought: `residual_claims` blanks every assert span and every permitted
non-asserting statement form and refuses on whatever is left.

THE CLAIM TABLE (closed; anything else raises `UnknownShape`):

  C1  assert!(<out>.status.success(), ..)            -> exit = "success"
  C2  assert!(!<out>.status.success(), ..)           -> exit = "failure"
  C3  assert!(<ok-binding>, ..)                      -> exit = "success"
  C4  assert!(!<ok-binding>, ..)                     -> exit = "failure"
  C5  assert_eq!(<stdout-expr>, <str>, ..)           -> stdout = <str>  (exact)
  C6  assert!(<stdout-expr>.is_empty(), ..)          -> stdout = ""     (exact)
  C7  assert!(<stderr-expr>.is_empty(), ..)          -> stderr = ""     (exact)
  C8  assert!(<stdout-expr>.contains(<str>), ..)     -> stdout_contains
  C9  assert!(<stderr-expr>.contains(<str>), ..)     -> stderr_contains
  C10 assert!(<stderr-expr>.contains(A)
              || <stdout-expr>.contains(A), ..)      -> rule 11, resolved
  C11 assert!((<stdout-expr>.clone() + &<stderr-expr>)
              .contains(A), ..)                      -> rule 11, resolved

`<str>` is a string literal OR an identifier that resolves, through the
evaluator's environment, to one -- a helper parameter fed a literal at its call
site is resolved, never pattern-matched by name. `<stdout-expr>`/`<stderr-expr>`
are `String::from_utf8_lossy(&<out>.<stream>)`, a `let`-binding of one, or a
tuple-destructured element of a helper's `(bool, String, String)` return; which
stream an expression names is resolved through the environment too, so
`stdout.contains(..)` in a file that binds `stdout` to *stderr* would be read
correctly. C10 and C11 are the two CROSS-STREAM shapes; both are presence
claims, both are resolved against the real binary by `captures.py` and pinned to
the stream that carries them (rule 11), and neither may be narrowed as an
absence claim (rule 2).

WHAT IS DELIBERATELY NOT A CLAIM. The second and later arguments of an
`assert!` are its PANIC MESSAGE, which the program under test never sees. A
`_`-prefixed helper parameter is a DEAD value (rule 2: `map_iteration_runtime`'s
`_expected` is computed at four call sites and asserted nowhere); it is recorded
as dead and must not become a claim.

FIXTURE AND ARGV RESOLUTION (rules 8/9). Every fixture body is produced by
`lexer.find_string_literals` off the source's own text -- the literal in a
builder fn's tail expression, or the literal at the call site -- never retyped.
`format!("main.{extension}")` is resolved by substituting the environment, which
is the only `format!` shape in the family and is asserted to be so.

  Usage:
    t19b4_extract.py            # census over every target; rc=1 on any
                                # unrecognised shape
    t19b4_extract.py <stem>     # one target, verbose
"""

from __future__ import annotations

import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.abspath(os.path.join(HERE, "..", ".."))
sys.path.insert(0, os.path.join(REPO, "tools/task-18-browser-pilot"))

from lexer import find_string_literals  # noqa: E402

TESTS = os.path.join(REPO, "crates/kali_cli/tests")

# ---------------------------------------------------------------------------
# THE WORK LIST, AND THE PREDICATE THAT PRODUCED IT
# ---------------------------------------------------------------------------
# Re-derived every run by `select()` below rather than trusted as a table
# (ruling 13): `STEMS` is asserted equal to the predicate's own answer, so a
# corpus change that moves the work list fails here instead of silently
# migrating a different set than the report describes.
STEMS = [
    "arena_reclamation_runtime",
    "for_await_frozen_set_map_constructor_result",
    "for_of_object_keys_iteration",
    "frozen_set_map_constructor_result",
    "growable_array_core",
    "map_iteration_runtime",
    "set_iteration_runtime",
    "template_literal_interpolation_runtime",
]

# The batch-3 runner signature. A target declaring it is batch 3's own shape and
# is left to a batch that takes that shape on its own terms -- batch 3's report
# §2 adjudicated `runtime_forin` exactly so, and re-deciding it here would be
# the per-file judgement the predicate exists to remove.
BATCH3_RUNNER = "fn run_source(src: &str) -> std::process::Output"

# TEXT-CLI disqualifiers. Each names a surface this batch's generator does not
# render, so a target carrying one needs a different instrument, not a wider
# shape table.
DISQUALIFY = {
    "env": r"\.env\(",
    "json_out": r'\.arg\(\s*"--output"',
    "bundle": r'\.arg\(\s*"--bundle"|\.arg\(\s*"--api"',
    "serde_json": r"serde_json",
    "stdin": r"\.stdin\(",
}


class UnknownShape(AssertionError):
    """A construct this extractor does not model.

    Raised rather than skipped. A forward extractor that skips what it does not
    understand converts a dropped claim into a green run, which is the failure
    this project keeps finding in its own instruments.
    """


# ---------------------------------------------------------------------------
# Lexical helpers (string-aware throughout; a `//` inside a JS fixture is
# program text and a `{` inside one is not a block)
# ---------------------------------------------------------------------------

def _spans(text: str) -> list[tuple[int, int]]:
    return [(l["start"], l["end"]) for l in find_string_literals(text)]


def blank_strings(text: str) -> str:
    """`text` with every string-literal INTERIOR replaced by spaces.

    Structure scanning (`fn`, `#[test]`, `const`) runs over this, because a JS
    fixture is full of `function main() {` and of braces that are program text,
    not Rust blocks. Offsets and line numbers are preserved exactly.
    """
    out = list(text)
    for a, b in _spans(text):
        for k in range(a, b):
            if out[k] != "\n":
                out[k] = " "
    return "".join(out)


def blank_line_comments(text: str) -> str:
    """`text` with `//` comment bodies replaced by spaces, string-aware.

    A commented-out `assert!` is not an assertion; a `//` inside a fixture is
    not a comment. Newlines are preserved so every offset still names its line.
    """
    spans = _spans(text)
    out = list(text)
    i, n, si = 0, len(text), 0
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


def _matching(text: str, open_idx: int, opener: str, closer: str) -> int:
    """Index just past the delimiter matching the one at `open_idx`."""
    spans = [(a + open_idx, b + open_idx)
             for a, b in _spans(text[open_idx:])]
    depth, i, n, si = 0, open_idx, len(text), 0
    while i < n:
        while si < len(spans) and spans[si][1] <= i:
            si += 1
        if si < len(spans) and spans[si][0] <= i < spans[si][1]:
            i = spans[si][1]
            continue
        if text[i] == opener:
            depth += 1
        elif text[i] == closer:
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    raise UnknownShape(f"unbalanced {opener!r} at offset {open_idx}")


def split_top_commas(text: str) -> list[str]:
    spans = _spans(text)
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
    return [p.strip() for p in parts if p.strip()]


def norm(s: str) -> str:
    return re.sub(r"\s+", " ", s).strip()


def split_statements(block: str) -> list[str]:
    """Top-level statements of a `{...}` body, string- and brace-aware.

    A statement ends at a `;` at depth 0, or at the `}` closing a depth-0
    block (a `for`/`if`). The trailing tail expression of a builder fn comes
    back as its own statement.
    """
    inner = block.strip()
    if inner.startswith("{"):
        inner = inner[1:-1]
    spans = _spans(inner)
    out, cur, depth, i, n, si = [], [], 0, 0, len(inner), 0
    while i < n:
        while si < len(spans) and spans[si][1] <= i:
            si += 1
        if si < len(spans) and spans[si][0] <= i < spans[si][1]:
            cur.append(inner[i:spans[si][1]])
            i = spans[si][1]
            continue
        c = inner[i]
        cur.append(c)
        if c in "([{":
            depth += 1
        elif c in ")]}":
            depth -= 1
            if depth == 0 and c == "}":
                # `} else {` and `} else if` continue the SAME statement; a
                # split here would file the else-arm as a top-level statement of
                # its own and evaluate it unconditionally.
                rest = inner[i + 1:]
                if re.match(r"\s*else\b", rest):
                    i += 1
                    continue
                out.append("".join(cur))
                cur = []
        elif c == ";" and depth == 0:
            out.append("".join(cur))
            cur = []
        i += 1
    if "".join(cur).strip():
        out.append("".join(cur))
    return [s.strip() for s in out if s.strip()]


def one_literal(frag: str, *, where: str) -> str:
    lits = find_string_literals(frag)
    if len(lits) != 1:
        raise UnknownShape(f"{where}: expected exactly one string literal in "
                           f"{norm(frag)[:100]!r}, found {len(lits)}")
    return lits[0]["value"]


# ---------------------------------------------------------------------------
# Values
# ---------------------------------------------------------------------------

class Str:
    __slots__ = ("value",)

    def __init__(self, value: str):
        self.value = value


class DirVal:
    """A directory. Identity matters, its NAME does not: the source builds temp
    directory names from a process id and a counter, and the runner gives every
    trial its own directory anyway. What the identity is FOR is U2: an
    invocation whose argv names files in two different source directories is
    flattened into one trial directory by the migration, and that flattening has
    to be measured rather than assumed inert."""
    __slots__ = ("uid",)
    _n = [0]

    def __init__(self):
        DirVal._n[0] += 1
        self.uid = DirVal._n[0]


class Opaque:
    """A value this extractor deliberately does not compute.

    Only ever legal as a directory NAME. Any other use raises: an opaque value
    reaching a fixture body or an assertion would put text into a case file that
    is not the source's text, which is a rule-9 violation by construction.
    """
    __slots__ = ("why",)

    def __init__(self, why: str):
        self.why = why


class PathVal:
    """A fixture path. `dir` is the directory the SOURCE put it in."""
    __slots__ = ("name", "dir")

    def __init__(self, name: str, dir: DirVal):
        self.name = name
        self.dir = dir


class OutputVal:
    __slots__ = ("inv",)

    def __init__(self, inv):
        self.inv = inv


class StreamVal:
    __slots__ = ("inv", "stream")

    def __init__(self, inv, stream: str):
        self.inv = inv
        self.stream = stream


class SuccessVal:
    __slots__ = ("inv",)

    def __init__(self, inv):
        self.inv = inv


class Invocation:
    """One real `kali` process the source runs."""

    def __init__(self, fn_name: str, where: str):
        self.fn_name = fn_name          # the `#[test]` fn it belongs to
        self.where = where
        self.argv: list = []            # tokens: str, or PathVal
        self.fixtures: dict[str, str] = {}
        self.claims: list[dict] = []
        self.dirs: set = set()
        self.order = 0

    def argv_tokens(self) -> list[str]:
        return [t.name if isinstance(t, PathVal) else t for t in self.argv]


# ---------------------------------------------------------------------------
# The source file
# ---------------------------------------------------------------------------

FN_RE = re.compile(
    r"(?:^|\n)fn\s+([a-z_][a-z0-9_]*)\s*\(([^)]*)\)\s*(?:->\s*([^{]+?))?\s*\{")
TEST_RE = re.compile(r"#\[test\]\s*\nfn\s+([a-z_][a-z0-9_]*)\s*\(\s*\)\s*\{")
CONST_RE = re.compile(r"(?:^|\n)(?:pub\s+)?const\s+([A-Z][A-Z0-9_]*)\s*:\s*&(?:'static\s+)?str\s*=")


class Source:
    def __init__(self, stem: str):
        self.stem = stem
        self.path = os.path.join(TESTS, stem + ".rs")
        self.text = open(self.path, encoding="utf-8").read()
        self.masked = blank_line_comments(self.text)
        self.skeleton = blank_strings(self.masked)
        self.fns: dict[str, dict] = {}
        self.consts: dict[str, str] = {}
        self.tests: list[dict] = []
        self._index()

    def _index(self):
        for m in CONST_RE.finditer(self.skeleton):
            tail = self.text[m.end():].lstrip()
            lits = find_string_literals(tail)
            if not lits or lits[0]["start"] != 0:
                raise UnknownShape(
                    f"{self.stem}.rs: `const {m.group(1)}` is not bound to a "
                    f"string literal")
            self.consts[m.group(1)] = lits[0]["value"]
        for m in FN_RE.finditer(self.skeleton):
            brace = m.end() - 1
            # `masked`, not `text`: a `{` inside a LINE COMMENT is not a block,
            # and `template_literal_interpolation_runtime.rs:67` carries exactly
            # one (`// \`\${\` would silently interpolate`) with no `}` to pair it.
            end = _matching(self.masked, brace, "{", "}")
            params = [p.strip() for p in m.group(2).split(",") if p.strip()]
            self.fns[m.group(1)] = {
                "name": m.group(1),
                "params": [p.split(":")[0].strip() for p in params],
                "ret": (m.group(3) or "").strip(),
                "body": self.text[brace:end],
                "masked": self.masked[brace:end],
                "start": brace,
                "line": self.text.count("\n", 0, m.start()) + 2,
            }
        for m in TEST_RE.finditer(self.skeleton):
            brace = m.end() - 1
            end = _matching(self.masked, brace, "{", "}")
            self.tests.append({
                "name": m.group(1),
                "body": self.text[brace:end],
                "masked": self.masked[brace:end],
                "start": brace,
                "end": end,
                "attr_line": self.text.count("\n", 0, m.start()) + 1,
            })
        if not self.tests:
            raise UnknownShape(f"{self.stem}.rs: no `#[test]` fn found")


# ---------------------------------------------------------------------------
# Expression evaluation
# ---------------------------------------------------------------------------

_IDENT = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*$")
_FORMAT = re.compile(r"^format!\s*\(")


class Evaluator:
    def __init__(self, src: Source):
        self.src = src
        self.invocations: list[Invocation] = []
        self.dead_values: list[tuple[str, str]] = []   # (where, literal)
        self._dirs: dict = {}
        self.opaque_helpers: set = set()
        self.inert_helpers: set = set()
        self.reached: dict[str, set] = {}              # fn name -> case names

    # -- expressions -------------------------------------------------------

    def ev(self, expr: str, env: dict, where: str):
        e = expr.strip()
        while e.startswith("&"):
            e = e[1:].strip()
        if e.startswith("(") and _matching(e, 0, "(", ")") == len(e):
            parts = split_top_commas(e[1:-1])
            if len(parts) > 1:
                return tuple(self.ev(p, env, where) for p in parts)
            return self.ev(e[1:-1], env, where)
        lits = find_string_literals(e)
        if lits and lits[0]["start"] == 0 and lits[0]["end"] == len(e):
            return Str(lits[0]["value"])
        if _IDENT.match(e):
            if e in env:
                return env[e]
            if e in self.src.consts:
                return Str(self.src.consts[e])
            raise UnknownShape(f"{where}: unbound identifier {e!r}")
        if _FORMAT.match(e):
            return Str(self._format(e, env, where))
        m = re.match(r"^([a-z_][a-z0-9_]*)\s*\(", e)
        if m and _matching(e, m.end() - 1, "(", ")") == len(e):
            name = m.group(1)
            if name in self.src.fns:
                args = split_top_commas(e[m.end():-1])
                return self.call(name, args, env, where)
            if name in ("tempdir",):
                return DirVal()
        m = re.match(r"^(.+?)\.expect\s*\(", e)
        if m:
            close = _matching(e, e.index(".expect(") + len(".expect"), "(", ")")
            if close == len(e):
                return self.ev(m.group(1), env, where)
        m = re.match(r"^String::from_utf8_lossy\s*\(", e)
        if m:
            inner = e[m.end() - 1:]
            close = _matching(inner, 0, "(", ")")
            arg = inner[1:close - 1].strip()
            rest = inner[close:].strip()
            if rest not in ("", ".into_owned()"):
                raise UnknownShape(f"{where}: unmodelled tail on from_utf8_lossy: {rest!r}")
            sm = re.match(r"^&?\s*([a-z_][a-z0-9_]*)\.(stdout|stderr)$", arg.strip())
            if not sm:
                raise UnknownShape(f"{where}: from_utf8_lossy over {arg!r}")
            base = env.get(sm.group(1))
            if not isinstance(base, OutputVal):
                raise UnknownShape(f"{where}: {sm.group(1)!r} is not a captured Output")
            return StreamVal(base.inv, sm.group(2))
        m = re.match(r"^([a-z_][a-z0-9_]*)\.status\.success\(\)$", e)
        if m:
            base = env.get(m.group(1))
            if not isinstance(base, OutputVal):
                raise UnknownShape(f"{where}: {m.group(1)!r} is not a captured Output")
            return SuccessVal(base.inv)
        if e == "std::env::temp_dir()":
            return DirVal()
        m = re.match(r"^(.*?)\.join\s*\(", e)
        if m:
            jopen = m.end() - 1
            close = _matching(e, jopen, "(", ")")
            if close != len(e):
                raise UnknownShape(f"{where}: unmodelled tail on join: {norm(e)[:80]!r}")
            base = self.ev(m.group(1), env, where)
            arg = self.ev(e[jopen + 1:close - 1], env, where)
            if isinstance(base, PathVal):
                base = self._as_dir(base)
            if not isinstance(base, DirVal):
                raise UnknownShape(f"{where}: `.join` on a non-directory")
            if isinstance(arg, Opaque):
                return DirVal()          # a uniquely-named sibling directory
            if not isinstance(arg, Str):
                raise UnknownShape(f"{where}: `.join` argument does not resolve")
            return PathVal(arg.value, base)
        m = re.match(r"^([a-z_][a-z0-9_]*)\.path\(\)$", e)
        if m:
            base = env.get(m.group(1))
            if isinstance(base, DirVal):
                return base
            raise UnknownShape(f"{where}: `.path()` on a non-directory")
        raise UnknownShape(f"{where}: unmodelled expression {norm(e)[:110]!r}")

    def _as_dir(self, p: PathVal) -> DirVal:
        key = ("dirof", p.dir.uid, p.name)
        if key not in self._dirs:
            self._dirs[key] = DirVal()
        return self._dirs[key]

    def _format(self, e: str, env: dict, where: str) -> str:
        close = _matching(e, e.index("(", 0), "(", ")")
        if close != len(e):
            raise UnknownShape(f"{where}: unmodelled tail on format!: {e!r}")
        args = split_top_commas(e[e.index("(") + 1:close - 1])
        tmpl = one_literal(args[0], where=where)
        positional = [self.ev(a, env, where) for a in args[1:]]
        out, i, pi = [], 0, 0
        while i < len(tmpl):
            c = tmpl[i]
            if c == "{" and i + 1 < len(tmpl) and tmpl[i + 1] == "{":
                out.append("{")
                i += 2
                continue
            if c == "}" and i + 1 < len(tmpl) and tmpl[i + 1] == "}":
                out.append("}")
                i += 2
                continue
            if c == "{":
                j = tmpl.index("}", i)
                spec = tmpl[i + 1:j]
                if spec == "":
                    val = positional[pi]
                    pi += 1
                elif _IDENT.match(spec):
                    val = self.ev(spec, env, where)
                else:
                    raise UnknownShape(f"{where}: unmodelled format spec {spec!r}")
                if not isinstance(val, Str):
                    raise UnknownShape(f"{where}: format! argument is not a string")
                out.append(val.value)
                i = j + 1
                continue
            out.append(c)
            i += 1
        return "".join(out)

    # -- calls -------------------------------------------------------------

    def call(self, name: str, args: list[str], env: dict, where: str):
        fn = self.src.fns[name]
        if name == "kali_bin":
            return Str("<kali>")
        if len(args) != len(fn["params"]):
            raise UnknownShape(
                f"{where}: `{name}` takes {len(fn['params'])} arg(s), called with {len(args)}")
        inner = {}
        for p, a in zip(fn["params"], args):
            val = self.ev(a, env, where)
            inner[p] = val
            if p.startswith("_") and isinstance(val, Str):
                # rule 2: a value computed at the call site and asserted nowhere
                # is not a claim and must never become one.
                self.dead_values.append((f"{where} -> {name}({p})", val.value))
        self.reached.setdefault(name, set()).add(env["__case__"])
        inner["__case__"] = env["__case__"]
        inner["__fn__"] = name
        inner["__pending__"] = env.setdefault("__pending__", {})
        try:
            return self.run_block(fn["masked"], inner, f"{where} -> {name}")
        except UnknownShape:
            # A helper this evaluator cannot execute is only tolerable when it
            # provably cannot observe the program under test and cannot write a
            # fixture -- then its value is a name, not a claim and not program
            # text. `Opaque` refuses every other use, so an unmodelled helper
            # can never put text this extractor invented into a case file.
            if self.inert(name):
                self.opaque_helpers.add(name)
                return Opaque(f"{name}() -- inert: sees no output, writes no fixture")
            raise

    _OBSERVES = (r"Command::new", r"\.status\b", r"\.stdout\b", r"\.stderr\b",
                 r"fs::write\s*\(", r"assert")

    def inert(self, name: str, seen: set | None = None) -> bool:
        """True iff `name` transitively neither observes a command's output nor
        writes a fixture nor asserts anything. DERIVED, not listed."""
        seen = seen or set()
        if name in seen:
            return True
        seen.add(name)
        fn = self.src.fns.get(name)
        if fn is None:
            return False
        body = fn["masked"]
        if any(re.search(rx, body) for rx in self._OBSERVES):
            return False
        for m in re.finditer(r"\b([a-z_][a-z0-9_]*)\s*\(", body):
            callee = m.group(1)
            if callee in self.src.fns and not self.inert(callee, seen):
                return False
        return True

    # -- statements --------------------------------------------------------

    def run_block(self, block: str, env: dict, where: str):
        tail = None
        for stmt in split_statements(block):
            tail = self.stmt(stmt, env, where)
        return tail

    def stmt(self, s: str, env: dict, where: str):
        st = s.strip().rstrip(";").strip()
        if not st:
            return None

        m = re.match(r"^for\s+([a-z_][a-z0-9_]*)\s+in\s+(\[[^\]]*\])\s*\{", st)
        if m:
            items = split_top_commas(m.group(2)[1:-1])
            body = st[st.index("{", m.end() - 1):]
            for it in items:
                env2 = dict(env)
                env2[m.group(1)] = self.ev(it, env, where)
                self.run_block(body, env2, where)
            return None

        m = re.match(r"^if\s+(.+?)\s*\{", st)
        if m and not st.startswith("if let"):
            cond = m.group(1).strip()
            brace = st.index("{", m.end() - 1)
            then_end = _matching(st, brace, "{", "}")
            then = st[brace:then_end]
            rest = st[then_end:].strip()
            els = None
            if rest.startswith("else"):
                r2 = rest[4:].strip()
                if not r2.startswith("{"):
                    raise UnknownShape(f"{where}: `else if` is not modelled")
                els = r2[:_matching(r2, 0, "{", "}")]
            taken = self.const_cond(cond, env, where)
            if taken:
                self.run_block(then, env, where)
            elif els is not None:
                self.run_block(els, env, where)
            return None

        m = re.match(r"^let\s+(?:mut\s+)?\(\s*([^)]*)\)\s*=\s*(.*)$", st, re.DOTALL)
        if m:
            names = [n.strip() for n in m.group(1).split(",")]
            fn_call = m.group(2).strip()
            got = self.ev(fn_call, env, where)
            if not isinstance(got, tuple) or len(got) != len(names):
                raise UnknownShape(f"{where}: tuple destructuring of {norm(fn_call)[:80]!r}")
            for n, v in zip(names, got):
                env[n] = v
            return None

        m = re.match(r"^let\s+(?:mut\s+)?([a-z_][a-z0-9_]*)\s*(?::\s*[^=]+)?=\s*(.*)$",
                     st, re.DOTALL)
        if m:
            name, rhs = m.group(1), m.group(2).strip()
            env[name] = self.rhs(rhs, env, where, name)
            return None

        if re.match(r"^assert(?:_eq)?!\s*\(", st):
            self.assertion(st, env, where)
            return None

        m = re.match(r"^([a-z_][a-z0-9_]*)\s*\(", st)
        if m and m.group(1) in self.src.fns and _matching(st, m.end() - 1, "(", ")") == len(st):
            self.call(m.group(1), split_top_commas(st[m.end():-1]), env, where)
            return None

        if re.match(r"^fs::create_dir_all\s*\(", st):
            return None

        m = re.match(r"^fs::write\s*\(", st)
        if m:
            close = _matching(st, m.end() - 1, "(", ")")
            args = split_top_commas(st[m.end():close - 1])
            if len(args) != 2:
                raise UnknownShape(f"{where}: fs::write with {len(args)} args")
            target = self.ev(args[0], env, where)
            body = self.ev(args[1], env, where)
            if not isinstance(target, PathVal) or not isinstance(body, Str):
                raise UnknownShape(f"{where}: fs::write over unmodelled operands")
            env.setdefault("__pending__", {})[(target.dir.uid, target.name)] = body.value
            return None

        # a tail expression (a builder fn's literal, or a returned tuple)
        try:
            return self.ev(st, env, where)
        except UnknownShape:
            raise UnknownShape(f"{where}: unmodelled statement {norm(st)[:110]!r}")

    def const_cond(self, cond: str, env: dict, where: str) -> bool:
        """`if` conditions are only modelled when they are constant at this call
        site -- which is what makes the branch a property of the CASE and not a
        runtime decision the case file would have to reproduce."""
        m = re.match(r"^([a-z_][a-z0-9_]*)\s*==\s*(.+)$", cond)
        if m:
            a = self.ev(m.group(1), env, where)
            b = self.ev(m.group(2), env, where)
            if isinstance(a, Str) and isinstance(b, Str):
                return a.value == b.value
        raise UnknownShape(f"{where}: `if {norm(cond)[:80]}` is not constant at this call site")

    def rhs(self, rhs: str, env: dict, where: str, name: str):
        r = rhs.strip()
        if r.startswith("Command::new") or r.startswith("std::process::Command::new"):
            return self.command(r, env, where)
        if r.startswith("tempdir("):
            return DirVal()
        return self.ev(r, env, where)

    # -- the process -------------------------------------------------------

    def command(self, expr: str, env: dict, where: str):
        head = re.match(r"^Command::new\s*\(|^std::process::Command::new\s*\(", expr)
        if not head:
            raise UnknownShape(f"{where}: unmodelled Command construction")
        close = _matching(expr, head.end() - 1, "(", ")")
        binname = expr[head.end():close - 1].strip()
        if binname not in ("kali_bin()",):
            raise UnknownShape(f"{where}: Command::new({binname!r}) is not the kali binary")
        inv = Invocation(env["__case__"], where)
        inv.order = len(self.invocations)
        rest = expr[close:]
        i = 0
        saw_output = False
        while i < len(rest):
            m = re.match(r"\s*\.\s*([a-z_]+)\s*\(", rest[i:])
            if not m:
                if rest[i:].strip() == "":
                    break
                raise UnknownShape(f"{where}: unmodelled Command tail {norm(rest[i:])[:80]!r}")
            meth = m.group(1)
            popen = i + m.end() - 1
            pclose = _matching(rest, popen, "(", ")")
            arg = rest[popen + 1:pclose - 1].strip()
            if meth == "arg":
                val = self.ev(arg, env, where)
                if isinstance(val, Str):
                    inv.argv.append(val.value)
                elif isinstance(val, PathVal):
                    inv.argv.append(val)
                else:
                    raise UnknownShape(f"{where}: `.arg({norm(arg)[:60]})` is neither")
            elif meth == "current_dir":
                pass          # the runner's trial dir is the cwd by construction
            elif meth == "output":
                saw_output = True
            elif meth == "expect":
                pass
            else:
                raise UnknownShape(f"{where}: unmodelled Command method `.{meth}()`")
            i = pclose
        if not saw_output:
            raise UnknownShape(f"{where}: a Command that is never `.output()`ed")
        pending = env.get("__pending__", {})
        for tok in inv.argv:
            if isinstance(tok, PathVal):
                key = (tok.dir.uid, tok.name)
                if key not in pending:
                    raise UnknownShape(
                        f"{where}: argv names {tok.name!r} but nothing wrote it")
                inv.fixtures[tok.name] = pending[key]
                inv.dirs.add(tok.dir.uid)
        # Every file the source wrote into a directory this command DRAWS FROM is
        # a sibling the command could see; one it wrote elsewhere is not. That
        # distinction is exactly U2's, and it is derived here rather than assumed.
        for (duid, nm), v in pending.items():
            if duid in inv.dirs:
                if nm in inv.fixtures and inv.fixtures[nm] != v:
                    raise UnknownShape(
                        f"{where}: two different files both named {nm!r} in one command")
                inv.fixtures.setdefault(nm, v)
        self.invocations.append(inv)
        return OutputVal(inv)

    # -- claims ------------------------------------------------------------

    def _stream_of(self, expr: str, env: dict, where: str):
        got = self.ev(expr, env, where)
        if isinstance(got, StreamVal):
            return got
        raise UnknownShape(f"{where}: {norm(expr)[:70]!r} is not a captured stream")

    def assertion(self, st: str, env: dict, where: str):
        m = re.match(r"^(assert_eq!|assert!)\s*\(", st)
        macro = m.group(1)
        close = _matching(st, m.end() - 1, "(", ")")
        if close != len(st.strip()):
            raise UnknownShape(f"{where}: unmodelled tail after an assertion")
        args = split_top_commas(st[m.end():close - 1])
        cond = args[0].strip()

        if macro == "assert_eq!":
            if len(args) < 2:
                raise UnknownShape(f"{where}: assert_eq! with one argument")
            left, right = args[0].strip(), args[1].strip()
            stream = self._stream_of(left, env, where)
            val = self.ev(right, env, where)
            if not isinstance(val, Str):
                raise UnknownShape(f"{where}: assert_eq! right side does not resolve")
            stream.inv.claims.append({
                "kind": "exact", "stream": stream.stream, "value": val.value,
                "shape": "C5", "text": norm(st)})
            return

        neg = cond.startswith("!")
        body = cond[1:].strip() if neg else cond

        # C1..C4 -- exit status, via an Output or a destructured success bool
        got = None
        try:
            got = self.ev(body, env, where)
        except UnknownShape:
            got = None
        if isinstance(got, SuccessVal):
            got.inv.claims.append({
                "kind": "exit", "value": "failure" if neg else "success",
                "shape": "C2" if neg else "C1", "text": norm(st)})
            return

        # C11 -- the COMBINED-STREAM shape (screen S26's widened arm)
        m = re.match(
            r"^\(\s*([a-z_][a-z0-9_.()]*?)\s*\.clone\(\)\s*\+\s*&\s*([a-z_][a-z0-9_]*)\s*\)"
            r"\s*\.contains\s*\(", body)
        if m:
            pclose = _matching(body, body.index(".contains(") + len(".contains"), "(", ")")
            if pclose != len(body):
                raise UnknownShape(f"{where}: unmodelled tail after a combined-stream contains")
            needle = self.ev(body[body.index(".contains(") + len(".contains("):pclose - 1],
                             env, where)
            a = self._stream_of(m.group(1), env, where)
            b = self._stream_of(m.group(2), env, where)
            if a.inv is not b.inv:
                raise UnknownShape(f"{where}: a combined claim over two different commands")
            if not isinstance(needle, Str):
                raise UnknownShape(f"{where}: combined-stream needle does not resolve")
            a.inv.claims.append({
                "kind": "cross_stream", "streams": [a.stream, b.stream],
                "value": needle.value, "shape": "C11", "text": norm(st)})
            return

        # C10 -- a cross-stream OR of the same needle
        parts = [p.strip() for p in re.split(r"\|\|", body)]
        if len(parts) > 1:
            resolved = []
            for p in parts:
                pm = re.match(r"^(.+?)\.contains\s*\(", p)
                if not pm:
                    raise UnknownShape(f"{where}: unmodelled disjunct {norm(p)[:70]!r}")
                pclose = _matching(p, p.index(".contains(") + len(".contains"), "(", ")")
                if pclose != len(p):
                    raise UnknownShape(f"{where}: unmodelled tail on a disjunct")
                needle = self.ev(p[p.index(".contains(") + len(".contains("):pclose - 1],
                                 env, where)
                stream = self._stream_of(pm.group(1), env, where)
                if not isinstance(needle, Str):
                    raise UnknownShape(f"{where}: disjunct needle does not resolve")
                resolved.append((stream, needle.value))
            invs = {s.inv for s, _ in resolved}
            if len(invs) != 1:
                raise UnknownShape(f"{where}: a disjunction over two different commands")
            needles = {v for _, v in resolved}
            if len(needles) != 1:
                raise UnknownShape(
                    f"{where}: a disjunction over DIFFERENT needles {sorted(needles)} -- "
                    f"ruling 17's two-degrees-of-freedom case, not this batch's shape")
            resolved[0][0].inv.claims.append({
                "kind": "cross_stream", "streams": [s.stream for s, _ in resolved],
                "value": needles.pop(), "shape": "C10", "text": norm(st)})
            return

        # C6/C7 -- an exactly-empty stream
        m = re.match(r"^(.+?)\.is_empty\(\)$", body)
        if m and not neg:
            stream = self._stream_of(m.group(1), env, where)
            stream.inv.claims.append({
                "kind": "exact", "stream": stream.stream, "value": "",
                "shape": "C6" if stream.stream == "stdout" else "C7",
                "text": norm(st)})
            return

        # C8/C9 -- a substring claim on one stream
        m = re.match(r"^(.+?)\.contains\s*\(", body)
        if m and not neg:
            pclose = _matching(body, body.index(".contains(") + len(".contains"), "(", ")")
            if pclose != len(body):
                raise UnknownShape(f"{where}: unmodelled tail after `.contains(...)`")
            needle = self.ev(body[body.index(".contains(") + len(".contains("):pclose - 1],
                             env, where)
            stream = self._stream_of(m.group(1), env, where)
            if not isinstance(needle, Str):
                raise UnknownShape(f"{where}: `.contains` needle does not resolve")
            stream.inv.claims.append({
                "kind": "contains", "stream": stream.stream, "value": needle.value,
                "shape": "C8" if stream.stream == "stdout" else "C9",
                "text": norm(st)})
            return

        # C3/C4 -- a destructured success bool
        if isinstance(env.get(body), SuccessVal):
            env[body].inv.claims.append({
                "kind": "exit", "value": "failure" if neg else "success",
                "shape": "C4" if neg else "C3", "text": norm(st)})
            return

        raise UnknownShape(
            f"{where}: assertion shape not in the table: {norm(st)[:140]!r}")


# ---------------------------------------------------------------------------
# Residual-claim closure (the table is closed over CLAIMS, not over macros)
# ---------------------------------------------------------------------------

# A form whose right-hand side TOUCHES the command's output is blanked only as
# far as the closing paren of that call, so anything CHAINED onto it stays in the
# residue -- batch 3 measured that distinction when blanking a lossy binding to
# its `;` swallowed a `.find(..).expect(..)` whole.
_PERMITTED_CALL = [
    r"String::from_utf8_lossy\s*\(",
]

# A statement whose right-hand side cannot carry a claim about the program under
# test, because the evaluator has already validated it far more strictly than
# this residue scan could. The `Command` chain is here for exactly that reason:
# `Evaluator.command` raises on any method outside `.arg`/`.current_dir`/
# `.output`/`.expect`, so a chained claim cannot reach this point.
_PERMITTED_STMT = [
    r"^let\s+(?:mut\s+)?[a-z_][a-z0-9_]*\s*=\s*(?:std::process::)?Command::new\s*\(",
    r"^let\s+(?:mut\s+)?[a-z_][a-z0-9_]*\s*=\s*tempdir\s*\(",
    r"^let\s+(?:mut\s+)?[a-z_][a-z0-9_]*\s*=\s*[a-z_][a-z0-9_]*\.path\(\)\.join\s*\(",
    r"^let\s+(?:mut\s+)?[a-z_][a-z0-9_]*\s*=\s*write_temp_[a-z_]*\s*\(",
    r"^let\s+(?:mut\s+)?[a-z_][a-z0-9_]*\s*=\s*std::env::(?:var|temp_dir)\s*\(",
    r"^let\s+(?:mut\s+)?[a-z_][a-z0-9_]*\s*=\s*[a-z_][a-z0-9_]*\.join\s*\(",
    r"^fs::write\s*\(",
    r"^fs::create_dir_all\s*\(",
    r"^static\s+[A-Z]",
    # A helper's TAIL TUPLE return -- the `(bool, String, String)` shape named in
    # the module docstring. It is not a claim: it hands the captured output to
    # the caller, where the assertions live and where the table reads them. The
    # evaluator has already resolved every member, so an unmodelled member could
    # not have reached this point.
    r"^\(\s*[a-z_][a-z0-9_]*\.status\.success\(\)\s*,",
]

_CLAIM_TOKENS = [
    (r"\bpanic!", "a bare `panic!` -- an assertion written as control flow"),
    (r"\.contains\s*\(", "a `.contains` outside any `assert*!`"),
    (r"\.expect\s*\(", "an `.expect(...)` -- a claim carried by an unwrap"),
    (r"\.unwrap\s*\(", "an `.unwrap()` -- a claim carried by an unwrap"),
    (r"\bdebug_assert", "a `debug_assert*!`"),
    (r"\bmatches!", "a `matches!` predicate"),
    (r"\bassert_ne!", "an `assert_ne!`"),
    (r"\.status\b", "a use of the exit status outside any `assert*!`"),
    (r"\.stdout\b", "a use of stdout outside any `assert*!`"),
    (r"\.stderr\b", "a use of stderr outside any `assert*!`"),
]


def _blank_span(s: str, lo: int, hi: int) -> str:
    return s[:lo] + "".join(c if c == "\n" else " " for c in s[lo:hi]) + s[hi:]


def residual_claims(src: "Source", fn: dict) -> list[str]:
    """What a body does that the claim table cannot see.

    Blanks every `assert*!` span, every permitted whole statement and every
    permitted call, then refuses on any surviving claim token. This is the
    difference between a table closed over MACROS and one closed over CLAIMS --
    batch 3 shipped the first and had to add the second in a fix round.
    """
    text = fn["masked"]
    for m in reversed(list(re.finditer(r"\bassert(?:_eq|_ne)?!\s*\(", text))):
        try:
            end = _matching(text, m.end() - 1, "(", ")")
        except UnknownShape:
            continue
        text = _blank_span(text, m.start(), end)
    # whole statements, recursing into `for`/`if` bodies -- a permitted form
    # inside a loop is still a permitted form, and a residue scan that stops at
    # the loop's own braces reports every statement in it.
    def blank_stmts(s: str, lo: int, hi: int) -> str:
        base = lo
        for st in split_statements(s[lo:hi]):
            idx = s.find(st, base, hi)
            if idx < 0:
                continue
            base = idx + len(st)
            if any(re.match(rx, st.strip()) for rx in _PERMITTED_STMT):
                s = _blank_span(s, idx, idx + len(st))
                continue
            if re.match(r"^(?:for|if|while|else)\b", st.strip()):
                j = idx
                while True:
                    k = s.find("{", j, idx + len(st))
                    if k < 0:
                        break
                    try:
                        end = _matching(s, k, "{", "}")
                    except UnknownShape:
                        break
                    s = blank_stmts(s, k + 1, end - 1)
                    j = end
        return s

    out_text = blank_stmts(text, 0, len(text))
    for pat in _PERMITTED_CALL:
        while True:
            m = re.search(pat, out_text)
            if not m:
                break
            i = out_text.index("(", m.end() - 1)
            try:
                end = _matching(out_text, i, "(", ")")
            except UnknownShape:
                break
            out_text = _blank_span(out_text, m.start(), end)
    out = []
    for rx, why in _CLAIM_TOKENS:
        for m in re.finditer(rx, out_text):
            line = fn.get("line_base", 0) + out_text.count("\n", 0, m.start())
            out.append(f"{why} @~{line}: "
                       f"{norm(out_text[max(0, m.start() - 40):m.start() + 40])!r}")
    return out


# ---------------------------------------------------------------------------
# Prose attribution, derived from source POSITION (rule 12 / U6)
# ---------------------------------------------------------------------------

def prose(src: Source) -> dict:
    """Split every comment into three populations by POSITION.

    * per-test    -- a paragraph inside a `#[test]` body, or directly abutting it
    * per-helper  -- a paragraph inside, or directly abutting, a non-test `fn`
    * file-wide   -- everything else

    Attribution follows the source's own layout rather than a per-comment
    judgement (U6), and a helper's prose reaches exactly the cases that helper
    produced -- which the evaluator records in `reached`.
    """
    sys.path.insert(0, os.path.join(REPO, "tools/task-18-browser-pilot"))
    from comment_coverage import extract_comment_paragraphs, extract_trailing_comments

    paras = []
    for line, lines in extract_comment_paragraphs(src.text):
        paras.append((line, lines))
    trailing = [(line, [t]) for line, t in extract_trailing_comments(src.text)]
    paras.extend(trailing)
    paras.sort()

    def line_of(off):
        return src.text.count("\n", 0, off) + 1

    # `head` is the item's OWN first line -- the `#[test]` attribute line for a
    # test, the `fn` line for a helper. A paragraph ABUTS the item when its LAST
    # line is `head - 1`. Written as `line + len(lines) == head` first, which is
    # off by one and filed every `///` doc block as file-wide; the symptom was
    # `comment_coverage.py` reporting a helper's doc "MISSING from ALL N cases",
    # which is what a rule-12 carry into the wrong place looks like.
    regions = []
    for t in src.tests:
        regions.append(("test", t["name"], line_of(t["start"]), line_of(t["end"]),
                        t["attr_line"]))
    for name, fn in src.fns.items():
        end = fn["start"] + len(fn["body"])
        regions.append(("fn", name, line_of(fn["start"]), line_of(end), fn["line"]))

    out = {"file": [], "test": {}, "fn": {}}
    for line, lines in paras:
        last = line + len(lines) - 1
        placed = False
        for kind, name, lo, hi, head in regions:
            if lo <= line <= hi:
                out[kind].setdefault(name, []).append((line, lines))
                placed = True
                break
        if not placed:
            for kind, name, lo, hi, head in regions:
                if last == head - 1:
                    out[kind].setdefault(name, []).append((line, lines))
                    placed = True
                    break
        if not placed:
            out["file"].append((line, lines))
    return out


# ---------------------------------------------------------------------------
# Driver
# ---------------------------------------------------------------------------

def extract(stem: str) -> dict:
    src = Source(stem)
    ev = Evaluator(src)
    per_test = []
    for t in src.tests:
        before = len(ev.invocations)
        env = {"__case__": t["name"], "__fn__": t["name"], "__pending__": {}}
        ev.run_block(t["masked"], env, f"{stem}.rs::{t['name']}")
        made = ev.invocations[before:]
        if not made:
            raise UnknownShape(f"{stem}.rs::{t['name']}: runs no kali command")
        for inv in made:
            if not inv.claims:
                raise UnknownShape(
                    f"{stem}.rs::{t['name']}: a command with no claim at all")
        t2 = dict(t)
        t2["line_base"] = t["attr_line"]
        res = residual_claims(src, t2)
        if res:
            raise UnknownShape(
                f"{stem}.rs::{t['name']}: claim(s) outside the table:\n    "
                + "\n    ".join(res))
        per_test.append({"name": t["name"], "invocations": made})
    for name, fn in src.fns.items():
        if name in ("kali_bin",):
            continue
        if ev.inert(name):
            # A helper that transitively neither observes a command's output nor
            # writes a fixture nor asserts anything cannot carry a claim ABOUT
            # THE PROGRAM UNDER TEST -- the only kind of claim a case file
            # carries. It can still carry the fixture-self-inspection shape
            # (ruling 10), which is not visible to any residue scan and is gated
            # by `find_fixture_self_inspection.py` instead; the generator runs
            # that tool over every target in this batch and requires it clean.
            ev.inert_helpers.add(name)
            continue
        f2 = dict(fn)
        f2["line_base"] = fn["line"]
        res = residual_claims(src, f2)
        if res:
            raise UnknownShape(
                f"{stem}.rs::{name} (helper): claim(s) outside the table:\n    "
                + "\n    ".join(res))
    return {"stem": stem, "src": src, "ev": ev, "tests": per_test}


def select() -> list[str]:
    """The work list, re-derived rather than tabulated (ruling 13)."""
    import glob
    import subprocess
    pat = re.compile(r"Migrated from tests/([A-Za-z0-9_/]+)\.rs")
    migrated = set()
    for p in glob.glob(os.path.join(TESTS, "cases/*/*.toml")):
        migrated |= set(pat.findall(open(p, encoding="utf-8").read()))
    clean = subprocess.run(
        [sys.executable, os.path.join(REPO, "tools/migration/screen_candidates.py"),
         "--list-clean"], capture_output=True, text=True, check=True).stdout.split()
    out = []
    for s in sorted(clean):
        if s in migrated:
            continue
        t = open(os.path.join(TESTS, s + ".rs"), encoding="utf-8").read()
        if BATCH3_RUNNER in t:
            continue
        if any(re.search(rx, t) for rx in DISQUALIFY.values()):
            continue
        out.append(s)
    return out


def main(argv: list[str]) -> int:
    if argv and argv[0] == "--list":
        print("\n".join(select()))
        return 0
    stems = argv or STEMS
    if not argv:
        derived = select()
        if derived != STEMS:
            print("WORK LIST MOVED -- the predicate no longer answers STEMS")
            print("  predicate:", derived)
            print("  STEMS    :", STEMS)
            return 1
    total_fns = total_inv = total_claims = 0
    for stem in stems:
        got = extract(stem)
        n_inv = sum(len(t["invocations"]) for t in got["tests"])
        n_cl = sum(len(i.claims) for t in got["tests"] for i in t["invocations"])
        kinds = {}
        for t in got["tests"]:
            for inv in t["invocations"]:
                for c in inv.claims:
                    kinds[c["shape"]] = kinds.get(c["shape"], 0) + 1
        total_fns += len(got["tests"])
        total_inv += n_inv
        total_claims += n_cl
        shapes = " ".join(f"{k}={v}" for k, v in sorted(kinds.items()))
        print(f"  {stem:<46}{len(got['tests']):>3} fn(s) {n_inv:>4} invocation(s) "
              f"{n_cl:>4} claim(s)   {shapes}")
        if argv:
            for t in got["tests"]:
                for inv in t["invocations"]:
                    print(f"      {t['name']}  argv={inv.argv_tokens()} "
                          f"fixtures={sorted(inv.fixtures)}")
                    for c in inv.claims:
                        print(f"         {c['shape']} {c['kind']} "
                              f"{c.get('stream', c.get('streams', ''))} {c.get('value','')!r}")
            for w, v in got["ev"].dead_values:
                print(f"      DEAD (rule 2) {w}: {v[:60]!r}")
    print(f"\n{len(stems)} target(s), {total_fns} `#[test]` fn(s), "
          f"{total_inv} invocation(s), {total_claims} claim(s)")
    print("EXTRACTOR CENSUS OK")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
