r"""Character-cursor Rust string-literal scanner for Task 18 pilot (browser/).

Written fresh for this task. Handles plain "..." strings (escapes: \n \t \r
\\ \" \' \0, \xHH, \u{HHHHH}, and the \<newline> continuation) and raw strings
r#*"..."#*. Used as the generator's fixture-copy mechanism -- every fixture
string embedded in a .toml is pulled through this, never hand-retyped.
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
        if c == 'r' and i + 1 < n and (text[i + 1] == '#' or text[i + 1] == '"') and (
            i == 0 or not (text[i - 1].isalnum() or text[i - 1] == '_')
        ):
            k = i + 1
            hashes = 0
            while k < n and text[k] == '#':
                hashes += 1
                k += 1
            if k < n and text[k] == '"':
                out.append((k, True, hashes))
                close = '"' + ('#' * hashes)
                end = text.find(close, k + 1)
                i = (end + len(close)) if end != -1 else n
                continue
        if c == '"':
            out.append((i, False, 0))
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
    for (qstart, is_raw, hashes) in _find_string_starts(text):
        if is_raw:
            close = '"' + ('#' * hashes)
            end = text.find(close, qstart + 1)
            if end == -1:
                raise ValueError("unterminated raw string")
            value = text[qstart + 1:end]
            full_end = end + len(close)
            raw_start = qstart - 1 - hashes
            results.append({'start': raw_start, 'end': full_end, 'value': value})
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
