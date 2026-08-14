r"""Character-cursor Rust string-literal scanner for Task 18 pilot (browser/).

Written fresh for this task. Handles plain "..." strings (escapes: \n \t \r
\\ \" \' \0, \xHH, \u{HHHHH}, and the \<newline> continuation) and raw strings
in ALL THREE prefixes rustc accepts -- r#*"..."#*, br#*"..."#*, cr#*"..."#*.
Used as the generator's fixture-copy mechanism -- every fixture string embedded
in a .toml is pulled through this, never hand-retyped.

`b"..."` and `c"..."` are ESCAPED literals, not raw, and take the plain-string
path. `rb"..."` is not a Rust prefix at all and is deliberately unhandled; see
`_find_string_starts` for the rustc transcript this rests on.
"""
import re


def _find_string_starts(text):
    i = 0
    n = len(text)
    out = []
    while i < n:
        c = text[i]
        if c == '/' and i + 1 < n and text[i + 1] == '/':
            j = text.find('\n', i)
            i = j if j != -1 else n
            continue
        # RAW-STRING PREFIXES, ASKED OF rustc RATHER THAN REMEMBERED. rustc
        # 1.97.1 accepts `r"x"`, `br"x"`, `br#"x"#`, `cr"x"`; `rb"x"` is
        # `error: prefix `rb` is unknown`, so it is deliberately absent. `b"x"`
        # and `c"x"` are ESCAPED literals, not raw, and fall through to the
        # plain-string branch below exactly as before.
        #
        # THE BUG THIS CLOSES (batch 2 fix round 5 §41, measured, disclosed, and
        # parked there). The guard was `c == 'r'` with a word-boundary check on
        # the character before it -- so for `br#"..."#` the preceding `b` IS an
        # identifier character, the guard fired, and the literal was never
        # recognised as a raw string. It did not merely miss it:
        #
        #   find_string_literals('... br#"json["stdout"].contains("X")"# ...')
        #   -> ['json[', '].contains(', ')']     # THREE literals INVENTED from
        #                                        # the raw string's interior
        #
        # The boundary check now sits before the whole prefix, which preserves
        # what it was there for: in `xbr"` the attempt at `b` sees `x` and the
        # attempt at `r` sees `b`, so both fail, and a word ending in `r`
        # (`"...operator"`) still cannot open a raw string.
        #
        # Direction: this module's consumers -- `check_fixtures.py`,
        # `fidelity.py`, every generator -- use it to DEMAND, so under-
        # recognition here goes loudly red rather than quietly green. That, plus
        # its blast radius over 161 irreplaceable browser case files, is why
        # batch 2 disclosed it instead of taking it. Closed here under the
        # controller's stated condition, with the condition measured: the full
        # census still reproduces all 161 byte-for-byte with 16 of 16 generators
        # at a fixed point, and the corpus differential moves no verdict.
        prefix = 0
        if c in 'bc' and i + 1 < n and text[i + 1] == 'r':
            prefix = 1
        head = i + prefix
        if text[head] == 'r' and head + 1 < n and (
            text[head + 1] == '#' or text[head + 1] == '"'
        ) and (
            i == 0 or not (text[i - 1].isalnum() or text[i - 1] == '_')
        ):
            k = head + 1
            hashes = 0
            while k < n and text[k] == '#':
                hashes += 1
                k += 1
            if k < n and text[k] == '"':
                # `i`, not a recomputed `k - 1 - hashes`: with a `b`/`c` prefix
                # the literal starts one byte earlier than the old arithmetic
                # assumed, and `start` is what `string_literals_in_range` and
                # every offset-based caller key on. Carried through rather than
                # re-derived, so the two cannot disagree.
                out.append((k, True, hashes, i))
                close = '"' + ('#' * hashes)
                end = text.find(close, k + 1)
                i = (end + len(close)) if end != -1 else n
                continue
        if c == '"':
            out.append((i, False, 0, i))
            j = i + 1
            while j < n:
                if text[j] == '\\':
                    j += 2
                    continue
                if text[j] == '"':
                    break
                j += 1
            i = j + 1
            continue
        i += 1
    return out


def _decode_plain(body):
    out = []
    i = 0
    n = len(body)
    while i < n:
        c = body[i]
        if c != '\\':
            out.append(c)
            i += 1
            continue
        if i + 1 >= n:
            raise ValueError("trailing backslash in string body")
        e = body[i + 1]
        if e == 'n':
            out.append('\n'); i += 2
        elif e == 't':
            out.append('\t'); i += 2
        elif e == 'r':
            out.append('\r'); i += 2
        elif e == '\\':
            out.append('\\'); i += 2
        elif e == '"':
            out.append('"'); i += 2
        elif e == "'":
            out.append("'"); i += 2
        elif e == '0':
            out.append('\0'); i += 2
        elif e == 'x':
            hexdigits = body[i + 2:i + 4]
            out.append(chr(int(hexdigits, 16)))
            i += 4
        elif e == 'u':
            assert body[i + 2] == '{'
            close = body.index('}', i + 3)
            hexdigits = body[i + 3:close]
            out.append(chr(int(hexdigits, 16)))
            i = close + 1
        elif e == '\n':
            j = i + 2
            while j < n and body[j] in ' \t\n\r':
                j += 1
            i = j
        else:
            raise ValueError(f"unhandled escape \\{e} in: {body[max(0,i-20):i+20]!r}")
    return ''.join(out)


def find_string_literals(text):
    results = []
    for (qstart, is_raw, hashes, lit_start) in _find_string_starts(text):
        if is_raw:
            close = '"' + ('#' * hashes)
            end = text.find(close, qstart + 1)
            if end == -1:
                raise ValueError("unterminated raw string")
            value = text[qstart + 1:end]
            full_end = end + len(close)
            results.append({'start': lit_start, 'end': full_end, 'value': value})
        else:
            j = qstart + 1
            n = len(text)
            while j < n:
                if text[j] == '\\':
                    j += 2
                    continue
                if text[j] == '"':
                    break
                j += 1
            body = text[qstart + 1:j]
            value = _decode_plain(body)
            results.append({'start': qstart, 'end': j + 1, 'value': value})
    return results


def string_literals_in_range(text, first_line, last_line):
    """1-indexed inclusive line range -> list of decoded string values, in
    source order, for every literal whose OPENING quote falls in that range."""
    lines = text.split('\n')
    offsets = [0]
    for line in lines:
        offsets.append(offsets[-1] + len(line) + 1)
    lo = offsets[first_line - 1]
    hi = offsets[last_line] if last_line < len(offsets) else len(text)
    out = []
    for lit in find_string_literals(text):
        if lo <= lit['start'] < hi:
            out.append(lit['value'])
    return out


def nth_literal(text, n):
    """0-indexed: the n-th string literal's decoded value in the whole file."""
    return find_string_literals(text)[n]['value']
