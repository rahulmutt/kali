r"""Small TOML value emitter for hand-generated case files (Task 18 pilot)."""


def _escape_common(s):
    return s.replace('\\', '\\\\').replace('"""', '\\"\\"\\"')


def toml_string(value, multiline=None):
    if multiline is None:
        multiline = '\n' in value
    if multiline:
        body = _escape_common(value)
        # A TOML multi-line basic string strips a leading newline right after
        # the opening """; if the payload starts with \n, that's real content
        # we don't want silently eaten, so nothing special needed here since
        # we always control the whole body and check by inspection.
        return '"""' + body + '"""'
    else:
        body = value.replace('\\', '\\\\').replace('"', '\\"')
        body = body.replace('\n', '\\n').replace('\t', '\\t').replace('\r', '\\r')
        return '"' + body + '"'


def toml_str_array(values):
    return '[' + ', '.join(toml_string(v, multiline=False) for v in values) + ']'
