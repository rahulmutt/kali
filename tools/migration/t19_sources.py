#!/usr/bin/env python3
"""One resolver for "what are the bytes of `crates/kali_cli/tests/<stem>.rs`".

Task 19's deletion removes 42 migrated sources. Four of the fourteen gates in
`scripts/test-gate.sh --gates-only` are generators that READ those sources to
re-derive their case files byte-for-byte, and every one of them opened the
working tree directly. With the sources gone they all raised
`FileNotFoundError` -- verified, not predicted: the deletion was rehearsed in a
dirty tree first and `gen_task19_batch{2,3,4,5}.py` failed in exactly that way.

Batch 8C hit the same wall on the browser family and solved it with
`case_emit.source_bytes`: the working tree when the file is there, a pinned
immutable ref when it is not. This is that resolver for the Task 19 tooling,
and it is deliberately ONE resolver rather than four -- 8C's report records a
near-miss where two resolvers keyed on different lines and each looked locally
correct.

WHY A RAISE AND NEVER AN EMPTY STRING. A generator that cannot find its source
must be told so. This project has already paid for the alternative twice: a
`citation_tiers` fix round where an unreadable ref was written into a blob
unchecked and the instrument carried on printing wrong figures, and 8C's
`submodule_paths` filtering on `is_file()` so that "I could not find them" read
to its callers as "there are none". A generator that silently stops covering a
deleted source is a false green, and a false green here means shipping a case
corpus nobody re-derived.

WHY THIS REF. `T19_DELETION_REF` is the commit the deletion was prepared at:
every one of the 42 sources still exists there, with the bytes the case files
were generated from. It is not "HEAD" resolved at runtime -- a ref derived from
HEAD moves when HEAD moves, which is the moving figure this project's ruling 11
forbids -- it is a literal sha, checked into the tree, that means the same thing
forever.
"""

from __future__ import annotations

import os
import re
import subprocess
import sys
import tempfile

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
TESTS_REL = "crates/kali_cli/tests"
TESTS = os.path.join(REPO, TESTS_REL)

# `cc76f5a918b8d3ca88019c7c0c657314b9ac3cca` -- the last commit before Task 19's
# deletion, where all 110 top-level `.rs` (the 42 deleted included) are present.
# See the module docstring for why this is a literal and not `git rev-parse HEAD`.
T19_DELETION_REF = "cc76f5a918b8d3ca88019c7c0c657314b9ac3cca"

_BLOB: dict[tuple[str, str], str | None] = {}
_MATERIALISED: dict[str, str] = {}


class SourceUnavailable(AssertionError):
    """The bytes cannot be produced. Distinct from "the ref is unreachable"."""


def _blob_at(ref: str, rel: str) -> str | None:
    """`<rel>` at `<ref>`, or None when the ref resolves but lacks the path.

    THE TWO FAILURES BELOW HAVE DIFFERENT REMEDIES AND MUST NOT SHARE A
    MESSAGE. An UNREACHABLE ref is a shallow clone, fixed by fetching history;
    a ref that resolves but does not carry the path means the file was already
    gone at that commit, and telling that caller to re-fetch sends it to
    re-derive a ref that was right all along. 8C's `case_emit._blob_at` records
    the same split for the same reason.
    """
    key = (ref, rel)
    if key not in _BLOB:
        if subprocess.run(["git", "rev-parse", "-q", "--verify", f"{ref}^{{commit}}"],
                          cwd=REPO, capture_output=True).returncode:
            raise SourceUnavailable(
                f"`{ref}` is not a commit reachable in this repository. This "
                "instrument needs FULL history: in CI, actions/checkout must be "
                "given `fetch-depth: 0` (a default checkout is shallow and "
                "cannot resolve it); locally, `git fetch --unshallow`.")
        got = subprocess.run(["git", "cat-file", "blob", f"{ref}:{rel}"],
                             cwd=REPO, capture_output=True, text=True)
        _BLOB[key] = got.stdout if got.returncode == 0 else None
    return _BLOB[key]


def source_text(stem: str, *, ref: str = T19_DELETION_REF, quiet: bool = False) -> str:
    """The `.rs` this generator reads: the working tree, or `ref` if it is gone."""
    path = os.path.join(TESTS, stem + ".rs")
    if os.path.exists(path):
        with open(path, encoding="utf-8") as fh:
            return fh.read()
    blob = _blob_at(ref, f"{TESTS_REL}/{stem}.rs")
    if blob is None:
        raise SourceUnavailable(
            f"{stem}.rs is absent from the working tree AND from {ref}. The ref "
            "must name a commit where the source still EXISTS -- a deletion "
            "commit's PARENT, not the deletion commit. Refusing to guess which "
            "blob this generator meant.")
    if not quiet:
        print(f"    reading {stem}.rs at the Task 19 deletion ref {ref[:10]}",
              file=sys.stderr)
    return blob


def source_path(stem: str, *, ref: str = T19_DELETION_REF, quiet: bool = False) -> str:
    """A path that EXISTS and holds those bytes -- for a subprocess gate.

    Some callers need a filename, not a string: `gen_task19_batch2` hands the
    source to `check_rationale_fn_names.py` and friends as argv. A deleted
    source is materialised once into a temp file and reused, so the gate a
    generator runs against a deleted source is the same gate, over the same
    bytes, that it ran against the file.
    """
    path = os.path.join(TESTS, stem + ".rs")
    if os.path.exists(path):
        return path
    if stem not in _MATERIALISED:
        text = source_text(stem, ref=ref, quiet=quiet)
        d = tempfile.mkdtemp(prefix="t19-source-")
        out = os.path.join(d, stem + ".rs")
        with open(out, "w", encoding="utf-8") as fh:
            fh.write(text)
        _MATERIALISED[stem] = out
    return _MATERIALISED[stem]


def available(stem: str, *, ref: str = T19_DELETION_REF) -> bool:
    """Can this stem's bytes be produced at all -- tree or ref?

    For a caller enumerating a candidate list that may name a target this
    deletion removed. `os.path.exists` is the wrong question once a source can
    legitimately live only in history, and answering it wrongly is how a work
    list quietly shrinks.
    """
    if os.path.exists(os.path.join(TESTS, stem + ".rs")):
        return True
    return _blob_at(ref, f"{TESTS_REL}/{stem}.rs") is not None


def deleted_stems(*, ref: str = T19_DELETION_REF) -> list[str]:
    """Top-level `.rs` stems present at `ref` and absent from the tree.

    DERIVED, NOT LISTED -- 8C's ruling 18 #1, and the same reason
    `case_emit.deleted_by_family_deletion` derives its answer instead of
    carrying a manifest: a list goes stale, and a list that goes stale here
    silently shrinks a control's population, which is the exact failure this
    whole deletion step is trying not to commit.

    Used by the SELFTEST arms of `screen_candidates.py`, so that the controls
    which bound the screen's over-blocking keep running against the four
    already-migrated targets Task 19 deleted. The screen's PRODUCTION corpus
    stays tree-only: what remains to migrate is a question about the tree.
    """
    listing = subprocess.run(
        ["git", "ls-tree", "--name-only", f"{ref}:{TESTS_REL}"],
        cwd=REPO, capture_output=True, text=True)
    if listing.returncode:
        raise SourceUnavailable(
            f"cannot list {TESTS_REL} at {ref}: {listing.stderr.strip()}")
    return sorted(
        n[:-3] for n in listing.stdout.split()
        if n.endswith(".rs") and not os.path.exists(os.path.join(TESTS, n)))


_REPRO_RE = re.compile(r"(?:^|(?<=[\s=]))crates/kali_cli/tests/([A-Za-z0-9_]+)\.rs")


def resolve_repro(cmd: str) -> tuple[str, list[str]]:
    """`(command, notes)` -- a printed reproduction command, made runnable.

    A case file's header prints the command that reproduces its gate verdict,
    and those commands name `crates/kali_cli/tests/<stem>.rs`. Task 19's
    deletion removes 42 of those, so 43 case files now print a command whose
    source argument no longer exists in the tree. `gen_task19_batch4` and
    `gen_task19_batch5` both CHECK that a printed command's paths exist and
    (batch 5) that the command exits with the declared rc, so both went red on
    the deletion -- which is the gate working, not a bug in the gate.

    THIS DOES NOT MAKE THE PRINTED TEXT TRUE AGAIN, AND MUST NOT BE READ AS IF
    IT DID. It substitutes the pinned blob for the deleted path so the gate
    still verifies the SUBSTANCE of the claim -- that this command, over the
    source as it stood at `T19_DELETION_REF`, reproduces the declared verdict --
    and it returns a note per substitution so every run says out loud which
    paths it had to resolve from history. The shipped prose still names a path a
    reader cannot `ls`. Rewriting those 43 headers to print
    `git show <ref>:<path> > /tmp/<stem>.rs` first is the real repair; it is
    recorded in the Task 19 deletion report as found-and-not-fixed, because it
    regenerates the prose of 43 shipped case files and that is a corpus change,
    not a deletion.

    The same shape as batch 8C, which resolved a deleted source's citations
    against a historical blob (`citation_sweep.sh`) rather than rewording them.
    """
    notes: list[str] = []

    def sub(m):
        stem = m.group(1)
        if os.path.exists(os.path.join(TESTS, stem + ".rs")):
            return m.group(0)
        if not available(stem):
            # Neither the tree nor the ref has it. Left EXACTLY as written, so
            # the caller's own existence check fires with the caller's own
            # message: a resolver that swallowed this would turn "this command
            # names nothing" into "this command names a temp file I invented".
            return m.group(0)
        path = source_path(stem, quiet=True)
        notes.append(f"{m.group(0)} is deleted; resolved at "
                     f"{T19_DELETION_REF[:10]} -> {path}")
        return path

    return _REPRO_RE.sub(sub, cmd), notes


def selftest() -> int:
    bad = 0

    def check(ok, label, extra=""):
        nonlocal bad
        print(f"  {'ok ' if ok else 'FAIL'} {label}{(' — ' + extra) if extra else ''}")
        if not ok:
            bad += 1

    # The pin resolves, and carries a source the deletion removes.
    blob = _blob_at(T19_DELETION_REF, f"{TESTS_REL}/runtime_forin.rs")
    check(blob is not None and "#[test]" in blob,
          "the deletion ref carries runtime_forin.rs with real tests",
          f"{len(blob or '')} bytes")

    # A file still in the tree comes from the tree, byte-for-byte.
    live = "browser_cdp_smoke"
    with open(os.path.join(TESTS, live + ".rs"), encoding="utf-8") as fh:
        check(source_text(live) == fh.read(),
              "a source still in the tree is read from the tree")

    # An unknown name raises, and raises the RIGHT thing -- it does not come
    # back as "" and it does not come back as an unreachable-ref complaint.
    try:
        source_text("zz_no_such_target_ever")
        check(False, "an unknown stem raises")
    except SourceUnavailable as exc:
        check("absent from the working tree AND from" in str(exc),
              "an unknown stem raises SourceUnavailable, naming both places")

    # ... and an unreachable ref is a DIFFERENT message, because it has a
    # different remedy.
    try:
        _blob_at("0" * 40, f"{TESTS_REL}/runtime_forin.rs")
        check(False, "an unreachable ref raises")
    except SourceUnavailable as exc:
        check("fetch-depth" in str(exc),
              "an unreachable ref raises the fetch-history message, not the "
              "missing-path one")

    # `source_path` produces a real file for a source that is NOT in the tree.
    #
    # THIS PROBE MUST CALL `source_path`. The version it replaces wrote a temp
    # file with `open()` and read it back with `open()`, under the label
    # "materialising a ref blob to a path round-trips byte-for-byte" -- it
    # exercised the standard library and not one line of the branch four
    # `--gates-only` generators now depend on. A probe that cannot fail when the
    # thing it names is broken is not a probe; it is the sentence above it,
    # retyped as code.
    #
    # The stem is DERIVED from `deleted_stems()` rather than named, so this arm
    # keeps selecting a source that is genuinely absent from the tree and the
    # materialisation branch is the one actually taken -- and if nothing is
    # absent, that is reported as a failure rather than passed over, because
    # then the branch has no input at all.
    gone = deleted_stems()
    stem = "runtime_forin" if "runtime_forin" in gone else (gone[0] if gone else None)
    if stem is None:
        check(False, "no source is absent from the working tree, so "
                     "`source_path`'s materialisation branch cannot be exercised")
    else:
        ancient = _blob_at(T19_DELETION_REF, f"{TESTS_REL}/{stem}.rs")
        got = source_path(stem, quiet=True)
        check(got != os.path.join(TESTS, stem + ".rs") and os.path.isfile(got),
              f"source_path({stem!r}) takes the MATERIALISATION branch and "
              f"returns a path that EXISTS", got)
        # Read defensively: if the arm above already failed because the branch
        # handed back a path that does not exist, this must report a FAIL and
        # not a traceback. A crash is not a verdict -- the same rule
        # `audit_corpus_sweep._require_a_verdict` exists to enforce.
        materialised = None
        if os.path.isfile(got):
            with open(got, encoding="utf-8") as fh:
                materialised = fh.read()
        check(materialised is not None and materialised == ancient,
              "the file source_path materialised holds the REF's bytes, "
              "byte-for-byte")
        check(source_path(stem, quiet=True) == got,
              "a second call reuses the same materialised file rather than "
              "writing a second one")

    # `resolve_repro` leaves a live path alone, and says so when it does not.
    live_cmd = f"python3 x.py crates/kali_cli/tests/{live}.rs a.toml"
    got, notes = resolve_repro(live_cmd)
    check(got == live_cmd and notes == [],
          "resolve_repro leaves a command naming a LIVE source untouched")
    got, notes = resolve_repro(
        "python3 x.py crates/kali_cli/tests/zz_not_a_target.rs a.toml")
    check(got.endswith("a.toml") and notes == [],
          "resolve_repro leaves a name it cannot resolve alone, rather than "
          "inventing a path")

    check(available("runtime_forin"), "available() is true for a pinned source")
    check(not available("zz_no_such_target_ever"),
          "available() is false for a name in neither place")

    print("SOURCES SELFTEST OK" if not bad else f"SOURCES SELFTEST FAILED — {bad}")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(selftest() if "--selftest" in sys.argv else
             (print(source_text(sys.argv[1])) or 0))
