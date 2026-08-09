#!/usr/bin/env python3
"""U14's `extra` direction, as a gate that can actually fail.

Rule 2 is "never invent a claim the source did not make". U14 names the
checkable invariant behind it: the `extra` side of a source-vs-TOML diff. It
also names the failure mode -- "a checker that computes `extra` and discards it
has disabled the gate that catches inventions" -- which is exactly what batch
4's `verify_pair.sh` did: it piped `fidelity.py` through `head -4` and never
read its exit status (fix round 1, I6).

`fidelity.py`'s raw string diff is the wrong instrument for this on its own: it
reports 38-66 `extra` entries per pair, almost all TOML structural keys, step
kinds, axis values, case names and rationale prose. A gate nobody can read is a
gate nobody reads.

So this works at CLAIM level instead, reusing audit-case-migration.py's own
`assertion_strings()` -- the fields the case runner actually turns into
assertions -- with the `[matrix]` expanded first, so `app.${ext}` is compared as
`app.js`/`app.ts`/... An extra claim is accepted only if it is:

  1. a claim the source makes (audit's `claims()`), or
  2. a string that appears verbatim somewhere in the source `.rs` -- covers
     helper arguments like "run", "main.ts" and env values like "node", which
     the source genuinely contains but which none of audit's five claim kinds
     extracts, or
  3. explicitly justified in the case file with a header line
         # EXTRA-OK: <python repr of the string> -- <why>
     which is how a deliberate live-captured exact pin (ruling 3's `.contains`
     against a JSON leaf) declares itself.

Anything else is an assertion whose text exists nowhere in the source and which
nobody justified -- an invention, or a typo in a pin. Exit 1.

Usage: check_extra_claims.py SOURCE.rs TARGET.toml
"""

import importlib.util
import itertools
import os
import re
import sys
import tomllib

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from submodules import read_with_submodules  # noqa: E402

_AUDIT = os.path.join(os.path.dirname(os.path.dirname(
    os.path.dirname(os.path.abspath(__file__)))), "scripts", "audit-case-migration.py")


def _in_source(value, rs_text):
    """Is this string present in the .rs, in EITHER spelling?

    audit-case-migration.py checks every literal claim in two spellings -- as
    written in Rust source (escapes intact, `a\\nb`) and fully unescaped (a real
    newline) -- because a TOML basic string and a Rust literal render the same
    text differently. This checker originally tested only the decoded form and
    so reported `'3\\n3'` as an unexplained extra on four files where the source
    spells it literally. Same bug class, same fix.
    """
    if value in rs_text:
        return True
    escaped = (value.replace("\\", "\\\\").replace("\n", "\\n")
                    .replace("\t", "\\t").replace("\r", "\\r"))
    return escaped in rs_text


def _audit_module():
    spec = importlib.util.spec_from_file_location("audit", _AUDIT)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def _substitute(obj, cell):
    if isinstance(obj, str):
        for k, v in cell.items():
            obj = obj.replace("${" + k + "}", v)
        return obj
    if isinstance(obj, list):
        return [_substitute(x, cell) for x in obj]
    if isinstance(obj, dict):
        return {k: _substitute(v, cell) for k, v in obj.items()}
    return obj


def expanded_assertion_strings(mod, toml_path):
    """Every claim-bearing string in the case file, with the matrix expanded."""
    doc = tomllib.load(open(toml_path, "rb"))
    axes = doc.get("matrix") or {}
    keys = list(axes)
    combos = [dict(zip(keys, c)) for c in itertools.product(*[axes[k] for k in keys])] or [{}]
    out = set()
    for cell in combos:
        out |= set(mod.assertion_strings(_substitute(doc, cell)))
    return out


def declared_extras(toml_path):
    """`# EXTRA-OK: <repr> -- <why>` header declarations."""
    ok = {}
    for line in open(toml_path):
        if not line.lstrip().startswith("#"):
            continue
        m = re.match(r"\s*#\s*EXTRA-OK:\s*(.+?)\s+--\s+(.*)$", line)
        if m:
            try:
                ok[eval(m.group(1), {"__builtins__": {}})] = m.group(2).strip()
            except Exception:
                print(f"  MALFORMED EXTRA-OK (value must be a python repr): {line.strip()}")
    return ok


def main(argv):
    if len(argv) != 2:
        raise SystemExit(__doc__)
    rs_path, toml_path = argv
    mod = _audit_module()
    # U10: for a `#[path = "..."] mod ...;` carrier, the top-level `.rs` holds
    # the helpers and the SUBMODULES hold every `#[test]` fn -- so the argv
    # tokens ("build", "main.js") and the literals this gate accepts as
    # "present verbatim in the source" mostly live one hop away. Reading the
    # carrier alone reported them all as unexplained inventions. Same
    # resolution `audit-case-migration.py` does for itself; a file with no
    # `mod` declaration is unaffected.
    rs_text = read_with_submodules(rs_path, mod)

    source_claims = {v for vals in mod.claims(rs_text).values() for v in vals}
    toml_claims = expanded_assertion_strings(mod, toml_path)
    declared = declared_extras(toml_path)

    extras = sorted(c for c in toml_claims if c not in source_claims)
    in_source_text, justified, unexplained = [], [], []
    for e in extras:
        if e in declared:
            justified.append(e)
        elif e and _in_source(e, rs_text):
            in_source_text.append(e)
        else:
            unexplained.append(e)

    print(f"{len(toml_claims)} claim string(s) in {os.path.basename(toml_path)}; "
          f"{len(extras)} not among the source's extracted claims")
    print(f"  {len(in_source_text)} present verbatim in the source .rs (helper args, env values)")
    print(f"  {len(justified)} declared via `# EXTRA-OK:`")
    for e in justified:
        print(f"      {e!r} -- {declared[e]}")
    for e in unexplained:
        print(f"  UNEXPLAINED EXTRA: {e!r}")
    if unexplained:
        print(f"EXTRA CHECK FAILED — {len(unexplained)} asserted string(s) appear nowhere in "
              "the source and are not declared. Rule 2: never invent a claim.")
        return 1
    print("EXTRA CHECK OK — every extra claim is in the source text or declared")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
