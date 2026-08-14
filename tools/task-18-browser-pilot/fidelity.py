r"""Bidirectional source-vs-TOML fidelity diff for the Task 18 pilot.

Independent of audit-case-migration.py's own extraction: this module uses a
regex-per-position dispatcher (line comment / block comment / raw string /
plain string) rather than a character-cursor scanner, and reads the TOML side
via tomllib directly (not audit-case-migration.py's assertion_strings()).
Prints BOTH missing and extra, per the task brief's explicit requirement.
"""
import re
import sys
import tomllib

_LINE_COMMENT = re.compile(r'//[^\n]*')
_BLOCK_COMMENT = re.compile(r'/\*.*?\*/', re.DOTALL)
# RAW-STRING OPENER, PREFIX-CORRECT. One instance of a class enumerated
# repo-wide by Task 19 batch 4 and gated by
# `inst2_probes.probe_raw_string_recogniser_class`. The lookbehind sat directly
# on the `r`, so for `br#"..."#` the preceding `b` IS an identifier character,
# the guard fired, and the literal was not recognised as raw at all. It did not
# merely miss it -- the `_PLAIN_STRING` branch below then INVENTED literals out
# of the raw string's interior:
#
#   find_string_literals('let x = br#"json["stdout"].contains("X")"#;')
#     -> ['json[', '].contains(', ')']        # three literals that do not exist
#
# The boundary now sits before the whole prefix, which preserves what it was
# there for: in `xbr"` the `b` sees `x` and the `r` sees `b`, so both fail, and a
# word ending in `r` still cannot open a raw string. A bare `b"`/`c"` is an
# ESCAPED literal and still takes the `_PLAIN_STRING` path.
_RAW_STRING = re.compile(r'(?<![A-Za-z0-9_])(?:br|cr|r)(#*)"(.*?)"\1', re.DOTALL)
_PLAIN_STRING = re.compile(r'"((?:\\.|[^"\\])*)"', re.DOTALL)
_ESCAPE_RE = re.compile(r'\\(x[0-9a-fA-F]{2}|u\{[0-9a-fA-F]+\}|\n[ \t\r\n]*|.)', re.DOTALL)


def _decode_escape(m):
    e = m.group(1)
    table = {'n': '\n', 't': '\t', 'r': '\r', '\\': '\\', '"': '"', "'": "'", '0': '\0'}
    if e and e[0] == '\n':
        return ''
    if e in table:
        return table[e]
    if e.startswith('x'):
        return chr(int(e[1:], 16))
    if e.startswith('u{'):
        return chr(int(e[2:-1], 16))
    raise ValueError(f"unhandled escape: {e!r}")


def _decode_plain(body):
    return _ESCAPE_RE.sub(_decode_escape, body)


def find_string_literals(text):
    out = []
    i = 0
    n = len(text)
    while i < n:
        m = _LINE_COMMENT.match(text, i)
        if m:
            i = m.end(); continue
        m = _BLOCK_COMMENT.match(text, i)
        if m:
            i = m.end(); continue
        m = _RAW_STRING.match(text, i)
        if m:
            out.append(m.group(2)); i = m.end(); continue
        m = _PLAIN_STRING.match(text, i)
        if m:
            out.append(_decode_plain(m.group(1))); i = m.end(); continue
        i += 1
    return out


# Boring / structural literals that never carry a claim worth diffing (arg
# names shared by argparse, tiny numeric/format punctuation, etc.) -- kept
# short and explicit so nothing is silently excluded without a reason.
BORING = {"", "\n", " ", ".", ",", "/", "-", "_", "0", "1"}


def source_claims(rs_paths):
    claims = set()
    for p in rs_paths:
        text = open(p, encoding="utf-8").read()
        for lit in find_string_literals(text):
            if lit not in BORING and len(lit) > 1:
                claims.add(lit)
    return claims


def _walk(value, out):
    if isinstance(value, dict):
        for k, v in value.items():
            if isinstance(k, str) and len(k) > 1:
                out.add(k)
            _walk(v, out)
    elif isinstance(value, list):
        for v in value:
            _walk(v, out)
    elif isinstance(value, str):
        if value not in BORING and len(value) > 1:
            out.add(value)


def toml_claims(toml_path):
    doc = tomllib.load(open(toml_path, "rb"))
    out = set()
    _walk(doc, out)
    return out


def diff(rs_paths, toml_paths):
    sc = source_claims(rs_paths)
    tc = set()
    for p in toml_paths:
        tc |= toml_claims(p)
    missing = sc - tc
    extra = tc - sc
    return sc, tc, missing, extra


if __name__ == "__main__":
    args = sys.argv[1:]
    sep = args.index("--")
    rs_paths = args[:sep]
    toml_paths = args[sep + 1:]
    sc, tc, missing, extra = diff(rs_paths, toml_paths)
    print(f"source claims: {len(sc)}  toml claims: {len(tc)}")
    print(f"MISSING (in source, not in toml): {len(missing)}")
    for m in sorted(missing):
        print("  MISSING:", repr(m[:120]))
    print(f"EXTRA (in toml, not in source): {len(extra)}")
    for e in sorted(extra):
        print("  EXTRA:", repr(e[:120]))
