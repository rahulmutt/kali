r"""Small TOML value emitter for hand-generated case files (Task 18 pilot)."""


def _escape_common(s):
    return s.replace('\\', '\\\\').replace('"""', '\\"\\"\\"')


def toml_string(value, multiline=None):
    if multiline is None:
        multiline = '\n' in value
    if multiline:
        body = _escape_common(value)
        # A TOML multi-line basic string strips a leading newline right after
        # the opening `"""`. That is real content when the payload starts with
        # one, and "we always control the whole body and check by inspection"
        # -- what this comment used to say -- is not a mechanism. Inspection
        # missed it: Task 19 batch 2's `heap_grow_runtime` fixtures both open
        # with a newline, and the emitted `[source]` bodies were a byte short of
        # the program the source wrote. `cargo test` cannot see that (a leading
        # blank line is inert JS) and neither can the audit (`[source]` is not
        # an assertion surface); `check_fixtures.py` caught it, which is the
        # rule-9 gate doing exactly its job. Spelling the newline as an escape
        # keeps it: TOML honours `\n` inside a multi-line basic string, and the
        # escape is not the delimiter-adjacent literal newline the stripping
        # rule is about.
        if body.startswith("\n"):
            body = "\\n" + body[1:]
        return '"""' + body + '"""'
    else:
        body = value.replace('\\', '\\\\').replace('"', '\\"')
        body = body.replace('\n', '\\n').replace('\t', '\\t').replace('\r', '\\r')
        return '"' + body + '"'


def toml_str_array(values):
    return '[' + ', '.join(toml_string(v, multiline=False) for v in values) + ']'
